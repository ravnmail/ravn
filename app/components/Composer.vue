<script lang="ts" setup>
import { Editor, EditorContent } from '@tiptap/vue-3'
import { MailKit } from '~/lib/editor/extensions/MailKit'
import BasicBubbleMenu from '~/lib/editor/menus/BasicBubbleMenu.vue'
import LinkBubbleMenu from '~/lib/editor/menus/LinkBubbleMenu.vue'
import ContentMenu from '~/lib/editor/menus/ContentMenu.vue'
import AIMenu from '~/lib/editor/menus/AIMenu.vue'
import { Button } from '~/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '~/components/ui/select'
import EmailAutocompleteInput from '~/components/EmailAutocompleteInput.vue'
import { Badge } from '~/components/ui/badge'
import { SimpleTooltip } from '~/components/ui/tooltip'
import type { EmailAddress, EmailDetail } from '~/types/email'
import type { SaveDraftRequest, SendFromAccountRequest } from '~/composables/useAccountEmail'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import { Input } from '~/components/ui/input'
import { useCorvus } from '~/composables/useCorvus'
import { getFileIconForMimeType } from '~/lib/utils/fileIcons'
import { marked } from 'marked'
import { Separator } from '~/components/ui/separator'

interface Props {
  draft?: EmailDetail
  replyTo?: EmailDetail
  isReplyAll?: boolean
  forward?: EmailDetail
  initialAccountId?: string
  initialContent?: string
}

const props = defineProps<Props>()

console.log(props.initialContent, props.draft)

const emit = defineEmits<{
  sent: []
  discarded: []
  saved: [draftId: string]
  change: []
}>()

const { t } = useI18n()
const { accounts } = useAccounts()
const {
  isSending,
  isSavingDraft,
  loadAccounts,
  sendFromAccount,
  saveDraft,
  deleteDraft,
  filesToAttachmentData,
} = useAccountEmail()

const { loadAttachmentsForForward } = useAttachments()
const {
  isGeneratingSubject,
  generateSubjectStreaming,
} = useCorvus()

const selectedAccountId = ref<string | null>(null)
const draft = ref<Partial<EmailDetail>>({
  to: [],
  cc: [],
  bcc: [],
  subject: '',
  body_html: props.initialContent || '\n',
  conversation_id: undefined,
})
const attachments = ref<File[]>([])
const forwardedAttachments = ref<AttachmentData[]>([])
const showCc = ref(false)
const showBcc = ref(false)
const validationErrors = ref<Array<string | CleanTranslation>>([])
const currentDraftId = ref<string | null>(null)
const autoSaveInterval = ref<ReturnType<typeof setInterval> | null>(null)
const hasUnsavedChanges = ref(false)
const lastSavedAt = ref<Date | null>(null)
const isDraggingOver = ref(false)
const dragCounter = ref(0)

// Helper to mark as changed
const markAsChanged = () => {
  hasUnsavedChanges.value = true
  emit('change')
}

watch(accounts, (newAccounts) => {
  if (newAccounts.length > 0 && !selectedAccountId.value) {
    const firstAccount = newAccounts[0]
    if (firstAccount) {
      selectedAccountId.value = String(firstAccount.id)
    }
  }
})

const selectedAccount = computed(() => {
  return accounts.value.find(a => String(a.id) === selectedAccountId.value)
})

const canSend = computed(() => {
  return (
    selectedAccountId.value !== null
    && (draft.value.to?.length ?? 0) > 0
    && !isSending.value
    && !isSavingDraft.value
  )
})

const editor = new Editor({
  extensions: [
    MailKit.configure({
      placeholder: t('composer.placeholders.default'),
    }),
  ],
  content: draft.value.body_html || '\n',
  onUpdate: ({ editor }) => {
    draft.value.body_html = editor.getHTML()
    markAsChanged()
  },
})

onMounted(async () => {
  await loadAccounts()

  if (props.draft) {
    initializeFromDraft(props.draft)
  } else if (props.replyTo) {
    initializeReply(props.replyTo)
  } else if (props.forward) {
    await initializeForward(props.forward)
  } else if (props.initialAccountId) {
    selectedAccountId.value = String(props.initialAccountId)
  } else if (accounts.value.length > 0) {
    const firstAccount = accounts.value[0]
    if (firstAccount) {
      selectedAccountId.value = String(firstAccount.id)
    }
  }

  startAutoSave()
})

onUnmounted(() => {
  stopAutoSave()
  editor?.destroy()
})

function initializeFromDraft(draftEmail: EmailDetail) {
  currentDraftId.value = draftEmail.id
  selectedAccountId.value = draftEmail.account_id
  draft.value = {
    to: draftEmail.to,
    cc: draftEmail.cc,
    bcc: draftEmail.bcc,
    subject: draftEmail.subject || '',
    body_html: draftEmail.body_html || '\n',
    conversation_id: draftEmail.conversation_id,
  }
  editor.commands.setContent(draft.value.body_html || '\n')
  showCc.value = (draft.value.cc?.length ?? 0) > 0
  showBcc.value = (draft.value.bcc?.length ?? 0) > 0
  hasUnsavedChanges.value = false
}

function toSimpleHtml(text?: string): string {
  if (!text) return ''

  return marked.parse(text)
}

function initializeReply(email: EmailDetail) {
  selectedAccountId.value = email.account_id
  const toAddresses = [email.from]

  if (props.isReplyAll) {
    const originalRecipients = [
      ...email.to,
      ...(email.cc || [])
    ]
    // Filter out the sender's own email from CC to prevent self-cc
    const senderEmail = selectedAccount.value?.email
    const ccSet = new Set(originalRecipients.filter(addr =>
      addr.address !== email.from.address &&
      (!senderEmail || addr.address !== senderEmail)
    ))
    const ccList = Array.from(ccSet)

    draft.value = {
      to: toAddresses,
      cc: ccList,
      bcc: [],
      subject: email.subject?.startsWith('Re:') ? email.subject : `Re: ${email.subject || ''}`,
      body_html: `
        ${toSimpleHtml(props.initialContent)}
        <p><br></p>
        <p>On ${new Date(email.sent_at || email.received_at).toLocaleString()}, ${email.from.name || email.from.address} wrote:</p>
        <blockquote style="margin-left: 1em; padding-left: 1em; border-left: 2px solid #ccc;">
          ${email.body_html || email.body_plain || ''}
        </blockquote>
      `,
      conversation_id: email.conversation_id,
    }
    showCc.value = true
  } else {
    draft.value = {
      to: toAddresses,
      cc: [],
      bcc: [],
      subject: email.subject?.startsWith('Re:') ? email.subject : `Re: ${email.subject || ''}`,
      body_html: `
        ${toSimpleHtml(props.initialContent)}
        <p><br></p>
        <p>On ${new Date(email.sent_at || email.received_at).toLocaleString()}, ${email.from.name || email.from.address} wrote:</p>
        <blockquote style="margin-left: 1em; padding-left: 1em; border-left: 2px solid #ccc;">
          ${email.body_html || email.body_plain || ''}
        </blockquote>
      `,
      conversation_id: email.conversation_id,
    }
  }

  editor.commands.setContent(draft.value.body_html || '')
  markAsChanged()
}

async function initializeForward(email: EmailDetail) {
  selectedAccountId.value = email.account_id

  const forwardedBody = `
    ${toSimpleHtml(props.initialContent)}
    <p><br></p>
    <p>---------- Forwarded message ---------</p>
    <p><strong>From:</strong> ${email.from.name || email.from.address}</p>
    <p><strong>Date:</strong> ${new Date(email.sent_at || email.received_at).toLocaleString()}</p>
    <p><strong>Subject:</strong> ${email.subject || ''}</p>
    <p><strong>To:</strong> ${email.to.map(e => e.name || e.address).join(', ')}</p>
    <p><br></p>
    ${email.body_html || email.body_plain || ''}
  `

  draft.value = {
    to: [],
    cc: [],
    bcc: [],
    subject: email.subject?.startsWith('Fwd:') ? email.subject : `Fwd: ${email.subject || ''}`,
    body_html: forwardedBody,
    conversation_id: email.conversation_id,
  }

  editor.commands.setContent(forwardedBody)

  if (email.has_attachments) {
    await loadForwardedAttachments(email.id)
  }

  markAsChanged()
}

async function loadForwardedAttachments(emailId: string) {
  forwardedAttachments.value = await loadAttachmentsForForward(emailId)
}

function startAutoSave() {
  autoSaveInterval.value = setInterval(async () => {
    if (hasUnsavedChanges.value && selectedAccountId.value) {
      await handleAutoSave()
    }
  }, 30000) // 30 seconds
}

function stopAutoSave() {
  if (autoSaveInterval.value) {
    clearInterval(autoSaveInterval.value)
    autoSaveInterval.value = null
  }
}

async function handleAutoSave() {
  if (!selectedAccountId.value)
    return

  try {
    const request: SaveDraftRequest = {
      account_id: selectedAccountId.value,
      draft_id: currentDraftId.value || undefined,
      to: draft.value.to ?? [],
      cc: draft.value.cc ?? [],
      bcc: draft.value.bcc ?? [],
      subject: draft.value.subject || '',
      body: editor.getHTML(),
      conversation_id: draft.value.conversation_id,
    }

    const response = await saveDraft(request)
    currentDraftId.value = response.draft_id
    hasUnsavedChanges.value = false
    lastSavedAt.value = new Date()
  } catch (e) {
    console.error('Auto-save failed:', e)
  }
}

const isValidEmail = (email: EmailAddress): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(email.address.trim())
}

const validateForm = (): boolean => {
  validationErrors.value = []

  if (!selectedAccountId.value) {
    validationErrors.value.push(t('composer.noAccountSelected'))
    return false
  }

  if ((draft.value.to?.length ?? 0) === 0) {
    validationErrors.value.push(t('composer.noRecipients'))
    return false
  }

  const toEmails = draft.value.to ?? []
  const ccEmails = draft.value.cc ?? []
  const bccEmails = draft.value.bcc ?? []

  const allEmails = [...toEmails, ...ccEmails, ...bccEmails]
  const invalidEmails = allEmails.filter(email => !isValidEmail(email))

  if (invalidEmails.length > 0) {
    validationErrors.value.push(`${t('composer.invalidEmail')}: ${invalidEmails.join(', ')}`)
    return false
  }

  return true
}


const emailsToAddresses = (emails: string[]): EmailAddress[] => {
  return emails.map(email => ({
    address: email.trim(),
    name: undefined,
  }))
}


async function handleSend() {
  if (!validateForm())
    return

  try {
    const userAttachmentData = await filesToAttachmentData(attachments.value)

    const allAttachments = [
      ...userAttachmentData,
      ...forwardedAttachments.value
    ]

    const request: SendFromAccountRequest = {
      account_id: selectedAccountId.value!,
      to: draft.value.to || [],
      cc: draft.value.cc || [],
      bcc: draft.value.bcc || [],
      subject: draft.value.subject || '',
      body: editor.getHTML(),
      attachments: allAttachments,
      draft_id: currentDraftId.value ? currentDraftId.value : undefined,
      conversation_id: draft.value.conversation_id,
    }

    await sendFromAccount(request)

    hasUnsavedChanges.value = false
    currentDraftId.value = null

    emit('sent')
  } catch (e) {
    console.error('Failed to send email:', e)
  }
}

async function handleSaveDraft() {
  if (!selectedAccountId.value) {
    validationErrors.value = [t('composer.noAccountSelected')]
    return
  }

  try {
    const request: SaveDraftRequest = {
      account_id: selectedAccountId.value,
      draft_id: currentDraftId.value || undefined,
      to: emailsToAddresses(draft.value.to?.map(e => e.address) || []),
      cc: emailsToAddresses(draft.value.cc?.map(e => e.address) || []),
      bcc: emailsToAddresses(draft.value.bcc?.map(e => e.address) || []),
      subject: draft.value.subject || '',
      body: editor.getHTML(),
      conversation_id: draft.value.conversation_id,
    }

    const response = await saveDraft(request)
    currentDraftId.value = response.draft_id
    hasUnsavedChanges.value = false
    lastSavedAt.value = new Date()

    emit('saved', response.draft_id)
  } catch (e) {
    console.error('Failed to save draft:', e)
  }
}

async function handleDiscard() {
  if (currentDraftId.value) {
    try {
      await deleteDraft(currentDraftId.value)
    } catch (e) {
      console.error('Failed to delete draft:', e)
    }
  }

  emit('discarded')
}

const fileInputRef = ref<HTMLInputElement | null>(null)

function handleAttachmentClick() {
  fileInputRef.value?.click()
}

function handleFileSelect(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files) {
    const newFiles = Array.from(input.files)
    attachments.value.push(...newFiles)
    markAsChanged()
    input.value = ''
  }
}

function removeAttachment(index: number) {
  attachments.value.splice(index, 1)
  markAsChanged()
}

function removeForwardedAttachment(index: number) {
  forwardedAttachments.value.splice(index, 1)
  markAsChanged()
}

function formatFileSize(bytes: number): string {
  if (bytes === 0)
    return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Math.round(bytes / Math.pow(k, i) * 100) / 100} ${sizes[i]}`
}


const handleDragEnter = (event: DragEvent) => {
  event.preventDefault()
  event.stopPropagation()
  dragCounter.value++

  if (event.dataTransfer?.types.includes('Files')) {
    isDraggingOver.value = true
  }
}

const handleDragOver = (event: DragEvent) => {
  event.preventDefault()
  event.stopPropagation()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy'
  }
}

const handleDragLeave = (event: DragEvent) => {
  event.preventDefault()
  event.stopPropagation()
  dragCounter.value--

  if (dragCounter.value === 0) {
    isDraggingOver.value = false
  }
}

const handleDrop = (event: DragEvent) => {
  event.preventDefault()
  event.stopPropagation()
  dragCounter.value = 0
  isDraggingOver.value = false

  const files = event.dataTransfer?.files
  if (files && files.length > 0) {
    const newFiles = Array.from(files)
    attachments.value.push(...newFiles)
    markAsChanged()
  }
}

async function handleGenerateSubject() {
  if (!draft.value.body_html || !selectedAccountId.value) {
    return
  }

  try {
    const toAddresses = draft.value.to?.map(e => e.address) || []
    const ccAddresses = draft.value.cc?.map(e => e.address) || []
    const bccAddresses = draft.value.bcc?.map(e => e.address) || []
    const recipientsList = [...toAddresses, ...ccAddresses, ...bccAddresses]
    const senderEmail = selectedAccount.value?.email || ''
    const isReply = !!props.replyTo

    const generatedText = await generateSubjectStreaming(
      editor.getHTML(),
      senderEmail,
      recipientsList,
      isReply,
      draft.value.subject || undefined
    )

    if (generatedText) {
      draft.value.subject = generatedText.trim()
      markAsChanged()
    }
  } catch (error) {
    console.error('Failed to generate subject:', error)
  }
}

</script>

<template>
  <div
    class="flex flex-col h-full bg-background w-full relative"
    @dragenter="handleDragEnter"
    @dragleave="handleDragLeave"
    @dragover="handleDragOver"
    @drop="handleDrop"
  >
    <div class="flex items-center justify-between pb-2">
      <div class="flex items-center gap-1 ml-auto">
        <SimpleTooltip
          :tooltip="`${$t('composer.saveDraft')}`"
          shortcut="mod + S"
        >
          <Button
            :disabled="isSavingDraft || isSending || !selectedAccountId"
            variant="ghost"
            @click="handleSaveDraft"
          >
            <Icon
              v-if="isSavingDraft"
              class="animate-spin"
              name="lucide:loader-2"
            />
            <Icon
              v-else
              name="lucide:save"
            />
          </Button>
        </SimpleTooltip>

        <SimpleTooltip
          :tooltip="$t('composer.discardDraft')"
          shortcut="Escape"
        >
          <Button
            :disabled="isSending || isSavingDraft"
            variant="ghost"
            @click="handleDiscard"
          >
            <Icon name="lucide:trash-2"/>
          </Button>
        </SimpleTooltip>

        <Separator
          orientation="vertical"
        />

        <SimpleTooltip
          :tooltip="$t('composer.send')"
          shortcut="mod + enter"
        >
          <Button
            :disabled="!canSend"
            variant="primary"
            @click="handleSend"
          >
            <Icon
              v-if="isSending"
              class="animate-spin"
              name="lucide:loader-2"
            />
            <Icon
              v-else
              name="lucide:send"
            />
            <span class="ml-2">{{ isSending ? $t('composer.sending') : $t('composer.send') }}</span>
          </Button>
        </SimpleTooltip>
      </div>
    </div>

    <div
      v-if="validationErrors.length > 0"
      class="py-2 bg-destructive-background/10 border-b border-destructive"
    >
      <div
        v-for="(error, index) in validationErrors"
        :key="index"
        class="text-sm text-destructive flex items-center gap-2"
      >
        <Icon
          class="w-4 h-4"
          name="lucide:alert-circle"
        />
        {{ error }}
      </div>
    </div>

    <div>
      <div class="flex items-start py-1">
        <label class="text-sm font-medium text-muted pt-2 w-16 flex-shrink-0">
          {{ $t('composer.from') }}
        </label>
        <Select v-model="selectedAccountId">
          <SelectTrigger>
            <SelectValue/>
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="account in accounts"
              :key="account.id"
              :value="account.id"
            >
              <div class="flex items-center gap-2">
                <span>{{ account.name }}</span><span class="opacity-60"> &lt;{{ account.email }}&gt;</span>
              </div>
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div class="flex items-start py-1">
        <label class="text-sm font-medium text-muted pt-2 w-16 flex-shrink-0">
          {{ $t('composer.to') }}
        </label>
        <div class="flex-1 flex items-center gap-2">
          <EmailAutocompleteInput
            :model-value="draft.to|| []"
            :placeholder="$t('composer.enterRecipient')"
            @update:model-value="(emails) => { draft.to = emails; hasUnsavedChanges = true }"
          />
          <div class="flex items-center gap-1">
            <Button
              v-if="!showCc"
              size="xs"
              tabindex="-1"
              variant="ghost"
              @click="showCc = true"
            >
              {{ $t('composer.showCc') }}
            </Button>
            <Button
              v-if="!showBcc"
              size="xs"
              tabindex="-1"
              variant="ghost"
              @click="showBcc = true"
            >
              {{ $t('composer.showBcc') }}
            </Button>
          </div>
        </div>
      </div>
      <div
        v-if="showCc"
        class="flex items-start py-1"
      >
        <label class="text-sm font-medium text-muted pt-2 w-16 flex-shrink-0">
          {{ $t('composer.cc') }}
        </label>
        <div class="flex-1 flex items-center gap-2">
          <EmailAutocompleteInput
            :model-value="draft.cc || []"
            :placeholder="$t('composer.enterRecipient')"
            @update:model-value="(emails) => { draft.cc = emails; hasUnsavedChanges = true }"
          />
          <Button
            size="xs"
            tabindex="-1"
            variant="ghost"
            @click="showCc = false"
          >
            <Icon
              class="w-3 h-3"
              name="lucide:x"
            />
          </Button>
        </div>
      </div>
      <div
        v-if="showBcc"
        class="flex items-start py-1"
      >
        <label class="text-sm font-medium text-muted pt-2 w-16 flex-shrink-0">
          {{ $t('composer.bcc') }}
        </label>
        <div class="flex-1 flex items-center gap-2">
          <EmailAutocompleteInput
            :model-value="draft.bcc || []"
            :placeholder="$t('composer.enterRecipient')"
            @update:model-value="(emails) => { draft.bcc = emails; hasUnsavedChanges = true }"
          />
          <Button
            size="xs"
            tabindex="-1"
            variant="ghost"
            @click="showBcc = false"
          >
            <Icon
              class="w-3 h-3"
              name="lucide:x"
            />
          </Button>
        </div>
      </div>
      <div class="flex items-center py-1">
        <label class="text-sm font-medium text-muted w-16 flex-shrink-0">
          {{ $t('composer.subject') }}
        </label>
        <div class="flex-1 flex relative">
          <Input
            v-model="draft.subject"
            :placeholder="$t('composer.subject')"
            class="pr-10"
            type="text"
            @input="hasUnsavedChanges = true"
          />
          <SimpleTooltip tooltip="Generate subject with AI">
            <button
              :disabled="isGeneratingSubject || !draft.body_html || !selectedAccountId"
              class="absolute right-2 top-2 text-ai hover:text-primary disabled:opacity-50 disabled:cursor-not-allowed"
              @click="handleGenerateSubject"
            >
              <Icon
                v-if="isGeneratingSubject"
                class="w-4 h-4 animate-spin"
                name="lucide:loader-2"
              />
              <Icon
                v-else
                class="w-4 h-4"
                name="ravn:raven"
              />
            </button>
          </SimpleTooltip>
        </div>
      </div>
    </div>

    <div
      v-if="attachments.length > 0 || forwardedAttachments.length > 0"
      class="my-1 p-2 bg-surface rounded"
    >
      <div class="flex items-center gap-2 mb-2">
        <Icon
          class="w-4 h-4 text-muted"
          name="lucide:paperclip"
        />
        <span class="text-sm font-medium text-muted">{{ $t('composer.attachments') }}</span>
      </div>
      <div class="flex flex-wrap gap-2">
        <Badge
          v-for="(file, index) in attachments"
          :key="`user-${index}`"
          class="flex items-center gap-2 pr-1"
          variant="secondary"
        >
          <Icon
            :name="getFileIconForMimeType(file.type, file.name)"
            :size="24"
            mode="ib"
          />
          <div class="flex-1">
            <span class="text-sm font-medium max-w-64 truncate">{{ file.name }}</span>
            <div class="text-xs opacity-60">{{ formatFileSize(file.size) }}</div>
          </div>
          <button
            :title="$t('composer.removeAttachment')"
            class="rounded p-0.5 transition-colors hover:text-primary"
            @click="removeAttachment(index)"
          >
            <Icon
              class="w-3 h-3"
              name="lucide:x"
            />
          </button>
        </Badge>
        <Badge
          v-for="(att, index) in forwardedAttachments"
          :key="`forwarded-${index}`"
          class="flex items-center gap-2 pr-1"
          variant="outline"
        >
          <Icon
            class="w-3 h-3"
            name="lucide:forward"
          />
          <span class="text-xs">{{ att.filename }} ({{ formatFileSize(att.content.length) }})</span>
          <button
            :title="$t('composer.removeAttachment')"
            class="hover:bg-destructive/20 rounded p-0.5 transition-colors"
            @click="removeForwardedAttachment(index)"
          >
            <Icon
              class="w-3 h-3"
              name="lucide:x"
            />
          </button>
        </Badge>
      </div>
    </div>

    <div class="py-1 min-h-[40px]">
      <div class="flex items-center justify-between w-full">
        <div class="flex-1">
          <Toolbar :editor="editor"/>
        </div>
        <SimpleTooltip :tooltip="$t('composer.addAttachment')">
          <Button
            size="sm"
            variant="ghost"
            @click="handleAttachmentClick"
          >
            <Icon name="lucide:paperclip"/>
          </Button>
        </SimpleTooltip>
      </div>
    </div>

    <div class="flex-1 overflow-auto bg-surface rounded">
      <div class="w-full h-full max-w-none">
        <editor-content
          :editor="editor"
          class="w-full h-full p-3 prose prose-sm max-w-none"
        />
      </div>
    </div>

    <input
      ref="fileInputRef"
      accept="*/*"
      class="hidden"
      multiple
      type="file"
      @change="handleFileSelect"
    >

    <ContentMenu :editor="editor"/>
    <AIMenu :editor="editor"/>
    <BasicBubbleMenu :editor="editor"/>
    <LinkBubbleMenu :editor="editor"/>
    <div
      v-if="isDraggingOver"
      class="absolute inset-0 bg-dialog-overlay/30 backdrop-blur-xs z-50 flex items-center justify-center pointer-events-none"
    >
      <div class="bg-background rounded-lg p-8 text-center">
        <Icon
          class="w-16 h-16 mx-auto text-primary"
          name="lucide:upload-cloud"
        />
        <p class="text-lg font-semibold text-primary">
          {{ $t('composer.dropFilesHere') }}
        </p>
        <p class="text-sm text-muted">
          {{ $t('composer.releaseToAttach') }}
        </p>
      </div>
    </div>
  </div>
</template>
