<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import { Editor, EditorContent } from '@tiptap/vue-3'
import { marked } from 'marked'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'

import EmailAutocompleteInput from '~/components/EmailAutocompleteInput.vue'
import { Badge } from '~/components/ui/badge'
import { Button } from '~/components/ui/button'
import { Input } from '~/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select'
import { Separator } from '~/components/ui/separator'
import { SimpleTooltip } from '~/components/ui/tooltip'
import type { SaveDraftRequest, SendFromAccountRequest } from '~/composables/useAccountEmail'
import type { ContactNote } from '~/composables/useCorvus'
import { useCorvus } from '~/composables/useCorvus'
import { MailKit } from '~/lib/editor/extensions/MailKit'
import AIMenu from '~/lib/editor/menus/AIMenu.vue'
import BasicBubbleMenu from '~/lib/editor/menus/BasicBubbleMenu.vue'
import ContentMenu from '~/lib/editor/menus/ContentMenu.vue'
import LinkBubbleMenu from '~/lib/editor/menus/LinkBubbleMenu.vue'
import { getFileIconForMimeType } from '~/lib/utils/fileIcons'
import type { Contact } from '~/types/contact'
import type { EmailAddress, EmailDetail } from '~/types/email'

interface Props {
  draft?: EmailDetail
  replyTo?: EmailDetail
  isReplyAll?: boolean
  forward?: EmailDetail
  initialAccountId?: string
  initialTo?: EmailAddress[]
  initialCc?: EmailAddress[]
  initialBcc?: EmailAddress[]
  initialSubject?: string
  initialBodyText?: string
  initialContent?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  sent: []
  discarded: []
  saved: [draftId: string]
  change: []
}>()

const { t } = useI18n()
const { accounts } = useAccounts()
const { settings } = useSettings()
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
const { isGeneratingSubject, generateSubjectStreaming } = useCorvus()

// Lazily resolved AI notes for the current recipients, fetched via invoke on demand.
const activeContactNotes = ref<ContactNote[]>([])

const resolveContactNotes = async (emails: string[]): Promise<ContactNote[]> => {
  const notes: ContactNote[] = []
  for (const email of emails.slice(0, 5)) {
    if (!email) continue
    try {
      const contact = await invoke<Contact | null>('get_contact_by_email', { email })
      if (contact?.ai_notes?.trim()) {
        notes.push({
          email: contact.email,
          display_name: contact.display_name,
          notes: contact.ai_notes,
        })
      }
    } catch (_) {
      // ignore per-contact errors silently
    }
  }
  return notes
}

// Re-resolve notes whenever the recipient list changes (debounced to avoid
// hammering the DB on every keystroke in the To/Cc fields).
let resolveNotesTimer: ReturnType<typeof setTimeout> | null = null
const scheduleResolveNotes = () => {
  if (resolveNotesTimer) clearTimeout(resolveNotesTimer)
  resolveNotesTimer = setTimeout(async () => {
    const emails = [
      ...(draft.value.to ?? []).map((a) => a.address),
      ...(draft.value.cc ?? []).map((a) => a.address),
    ]
    activeContactNotes.value = await resolveContactNotes(emails)
  }, 400)
}

const selectedAccountId = ref<string | null>(null)

function plainTextToHtml(text?: string): string {
  if (!text) return ''

  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')

  return escaped
    .split(/\r\n|\r|\n/)
    .map(line => `<p>${line || '<br>'}</p>`)
    .join('')
}

const initialBodyHtml = computed(() => {
  if (props.initialBodyText) {
    return plainTextToHtml(props.initialBodyText)
  }

  return props.initialContent || '\n'
})

const draft = ref<Partial<EmailDetail>>({
  to: [...(props.initialTo ?? [])],
  cc: [...(props.initialCc ?? [])],
  bcc: [...(props.initialBcc ?? [])],
  subject: props.initialSubject || '',
  body_html: initialBodyHtml.value,
  conversation_id: undefined,
})

// Trigger note resolution whenever recipients change
watch(
  () => [draft.value.to, draft.value.cc],
  () => scheduleResolveNotes(),
  { deep: true }
)
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
  return accounts.value.find((a) => String(a.id) === selectedAccountId.value)
})

const canSend = computed(() => {
  return (
    selectedAccountId.value !== null &&
    (draft.value.to?.length ?? 0) > 0 &&
    !isSending.value &&
    !isSavingDraft.value
  )
})

const editor = new Editor({
  extensions: [
    MailKit.configure({
      placeholder: t('composer.placeholders.default'),
      settings: settings.value,
      emailContext: {
        sender: () => selectedAccount.value?.email ?? '',
        subject: () => draft.value.subject ?? '',
        isReply: () => !!props.replyTo,
        recipients: () => [
          ...(draft.value.to ?? []).map((a) => a.address),
          ...(draft.value.cc ?? []).map((a) => a.address),
        ],
        priorEmail: () => props.replyTo?.body_plain ?? props.replyTo?.body_html ?? undefined,
        contactNotes: () => activeContactNotes.value,
      },
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

  showCc.value = (draft.value.cc?.length ?? 0) > 0
  showBcc.value = (draft.value.bcc?.length ?? 0) > 0

  startAutoSave()

  // Focus the editor/composer on mount
  editor.commands.focus()
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

  const originalDate = new Date(email.sent_at || email.received_at).toLocaleString()
  const originalFrom = email.from.name || email.from.address
  const quotedBody = email.body_html || email.body_plain || ''

  let initialBodyContent = toSimpleHtml(props.initialContent)
  if (initialBodyContent && initialBodyContent !== '\n') {
    initialBodyContent = `${initialBodyContent}<p><br></p>`
  } else {
    initialBodyContent = '<p><br></p>'
  }

  if (props.isReplyAll) {
    const originalRecipients = [...email.to, ...(email.cc || [])]
    const senderEmail = selectedAccount.value?.email
    const ccSet = new Set(
      originalRecipients.filter(
        (addr) =>
          addr.address !== email.from.address && (!senderEmail || addr.address !== senderEmail)
      )
    )
    const ccList = Array.from(ccSet)

    draft.value = {
      to: toAddresses,
      cc: ccList,
      bcc: [],
      subject: email.subject?.startsWith('Re:') ? email.subject : `Re: ${email.subject || ''}`,
      body_html: initialBodyContent,
      conversation_id: email.conversation_id,
    }
    showCc.value = true
  } else {
    draft.value = {
      to: toAddresses,
      cc: [],
      bcc: [],
      subject: email.subject?.startsWith('Re:') ? email.subject : `Re: ${email.subject || ''}`,
      body_html: initialBodyContent,
      conversation_id: email.conversation_id,
    }
  }

  editor.commands.setContent(initialBodyContent)

  editor.commands.setQuotedContent(
    {
      type: 'reply',
      originalFrom,
      originalDate,
    },
    quotedBody
  )

  markAsChanged()
}

async function initializeForward(email: EmailDetail) {
  selectedAccountId.value = email.account_id

  const originalDate = new Date(email.sent_at || email.received_at).toLocaleString()
  const originalFrom = email.from.name || email.from.address
  const originalSubject = email.subject || ''
  const originalTo = email.to.map((e) => e.name || e.address).join(', ')
  const quotedBody = email.body_html || email.body_plain || ''

  let initialBodyContent = toSimpleHtml(props.initialContent)
  if (initialBodyContent && initialBodyContent !== '\n') {
    initialBodyContent = `${initialBodyContent}<p><br></p>`
  } else {
    initialBodyContent = '<p><br></p>'
  }

  draft.value = {
    to: [],
    cc: [],
    bcc: [],
    subject: email.subject?.startsWith('Fwd:') ? email.subject : `Fwd: ${email.subject || ''}`,
    body_html: initialBodyContent,
    conversation_id: email.conversation_id,
  }

  editor.commands.setContent(initialBodyContent)

  editor.commands.setQuotedContent(
    {
      type: 'forward',
      originalFrom,
      originalDate,
      originalSubject,
      originalTo,
    },
    quotedBody
  )

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
  if (!selectedAccountId.value) return

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
  const invalidEmails = allEmails.filter((email) => !isValidEmail(email))

  if (invalidEmails.length > 0) {
    validationErrors.value.push(`${t('composer.invalidEmail')}: ${invalidEmails.join(', ')}`)
    return false
  }

  return true
}

const emailsToAddresses = (emails: string[]): EmailAddress[] => {
  return emails.map((email) => ({
    address: email.trim(),
    name: undefined,
  }))
}

async function handleSend() {
  if (!validateForm()) return

  try {
    const userAttachmentData = await filesToAttachmentData(attachments.value)

    const allAttachments = [...userAttachmentData, ...forwardedAttachments.value]

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
      to: emailsToAddresses(draft.value.to?.map((e) => e.address) || []),
      cc: emailsToAddresses(draft.value.cc?.map((e) => e.address) || []),
      bcc: emailsToAddresses(draft.value.bcc?.map((e) => e.address) || []),
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
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Math.round((bytes / Math.pow(k, i)) * 100) / 100} ${sizes[i]}`
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
    const toAddresses = draft.value.to?.map((e) => e.address) || []
    const ccAddresses = draft.value.cc?.map((e) => e.address) || []
    const bccAddresses = draft.value.bcc?.map((e) => e.address) || []
    const recipientsList = [...toAddresses, ...ccAddresses, ...bccAddresses]
    const senderEmail = selectedAccount.value?.email || ''
    const isReply = !!props.replyTo

    const generatedText = await generateSubjectStreaming(
      editor.getHTML(),
      senderEmail,
      recipientsList,
      isReply,
      draft.value.subject || undefined,
      activeContactNotes.value.length > 0 ? activeContactNotes.value : undefined
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
    class="relative flex h-full w-full flex-col bg-background"
    @dragenter="handleDragEnter"
    @dragleave="handleDragLeave"
    @dragover="handleDragOver"
    @drop="handleDrop"
  >
    <div class="flex items-center justify-between pb-2">
      <div class="ml-auto flex items-center gap-1">
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
            <Icon name="lucide:trash-2" />
          </Button>
        </SimpleTooltip>

        <Separator orientation="vertical" />

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
      class="border-b border-destructive bg-destructive-background/10 py-2"
    >
      <div
        v-for="(error, index) in validationErrors"
        :key="index"
        class="flex items-center gap-2 text-sm text-destructive"
      >
        <Icon
          class="h-4 w-4"
          name="lucide:alert-circle"
        />
        {{ error }}
      </div>
    </div>

    <div>
      <div class="flex items-start py-1">
        <label class="w-16 shrink-0 pt-2 text-sm font-medium text-muted">
          {{ $t('composer.from') }}
        </label>
        <Select v-model="selectedAccountId">
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="account in accounts"
              :key="account.id"
              :value="account.id"
            >
              <div class="flex items-center gap-2">
                <span>{{ account.name }}</span
                ><span class="opacity-60"> &lt;{{ account.email }}&gt;</span>
              </div>
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div class="flex items-start py-1">
        <label class="w-16 shrink-0 pt-2 text-sm font-medium text-muted">
          {{ $t('composer.to') }}
        </label>
        <div class="flex flex-1 items-center gap-2">
          <EmailAutocompleteInput
            :model-value="draft.to || []"
            :placeholder="$t('composer.enterRecipient')"
            @update:model-value="
              (emails) => {
                draft.to = emails
                hasUnsavedChanges = true
              }
            "
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
        <label class="w-16 shrink-0 pt-2 text-sm font-medium text-muted">
          {{ $t('composer.cc') }}
        </label>
        <div class="flex flex-1 items-center gap-2">
          <EmailAutocompleteInput
            :model-value="draft.cc || []"
            :placeholder="$t('composer.enterRecipient')"
            @update:model-value="
              (emails) => {
                draft.cc = emails
                hasUnsavedChanges = true
              }
            "
          />
          <Button
            size="xs"
            tabindex="-1"
            variant="ghost"
            @click="showCc = false"
          >
            <Icon
              class="h-3 w-3"
              name="lucide:x"
            />
          </Button>
        </div>
      </div>
      <div
        v-if="showBcc"
        class="flex items-start py-1"
      >
        <label class="w-16 shrink-0 pt-2 text-sm font-medium text-muted">
          {{ $t('composer.bcc') }}
        </label>
        <div class="flex flex-1 items-center gap-2">
          <EmailAutocompleteInput
            :model-value="draft.bcc || []"
            :placeholder="$t('composer.enterRecipient')"
            @update:model-value="
              (emails) => {
                draft.bcc = emails
                hasUnsavedChanges = true
              }
            "
          />
          <Button
            size="xs"
            tabindex="-1"
            variant="ghost"
            @click="showBcc = false"
          >
            <Icon
              class="h-3 w-3"
              name="lucide:x"
            />
          </Button>
        </div>
      </div>
      <div class="flex items-center py-1">
        <label class="w-16 shrink-0 text-sm font-medium text-muted">
          {{ $t('composer.subject') }}
        </label>
        <div class="relative flex flex-1">
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
              class="absolute top-2 right-2 text-ai hover:text-primary disabled:cursor-not-allowed disabled:opacity-50"
              @click="handleGenerateSubject"
            >
              <Icon
                v-if="isGeneratingSubject"
                class="h-4 w-4 animate-spin"
                name="lucide:loader-2"
              />
              <Icon
                v-else
                class="h-4 w-4"
                name="ravn:raven"
              />
            </button>
          </SimpleTooltip>
        </div>
      </div>
    </div>

    <div
      v-if="attachments.length > 0 || forwardedAttachments.length > 0"
      class="my-1 rounded bg-surface p-2"
    >
      <div class="mb-2 flex items-center gap-2">
        <Icon
          class="h-4 w-4 text-muted"
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
            <span class="max-w-64 truncate text-sm font-medium">{{ file.name }}</span>
            <div class="text-xs opacity-60">{{ formatFileSize(file.size) }}</div>
          </div>
          <button
            :title="$t('composer.removeAttachment')"
            class="rounded p-0.5 transition-colors hover:text-primary"
            @click="removeAttachment(index)"
          >
            <Icon
              class="h-3 w-3"
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
            class="h-3 w-3"
            name="lucide:forward"
          />
          <span class="text-xs">{{ att.filename }} ({{ formatFileSize(att.content.length) }})</span>
          <button
            :title="$t('composer.removeAttachment')"
            class="rounded p-0.5 transition-colors hover:bg-destructive/20"
            @click="removeForwardedAttachment(index)"
          >
            <Icon
              class="h-3 w-3"
              name="lucide:x"
            />
          </button>
        </Badge>
      </div>
    </div>

    <div class="min-h-10 py-1">
      <div class="flex w-full items-center justify-between">
        <div class="flex-1">
          <Toolbar :editor="editor" />
        </div>
        <SimpleTooltip :tooltip="$t('composer.addAttachment')">
          <Button
            size="sm"
            variant="ghost"
            @click="handleAttachmentClick"
          >
            <Icon name="lucide:paperclip" />
          </Button>
        </SimpleTooltip>
      </div>
    </div>

    <div class="flex-1 overflow-auto rounded bg-surface">
      <div class="h-full w-full max-w-none">
        <editor-content
          :editor="editor"
          class="prose prose-sm h-full w-full max-w-none p-3"
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
    />

    <ContentMenu :editor="editor" />
    <AIMenu :editor="editor" />
    <BasicBubbleMenu :editor="editor" />
    <LinkBubbleMenu :editor="editor" />
    <div
      v-if="isDraggingOver"
      class="pointer-events-none absolute inset-0 z-50 flex items-center justify-center bg-dialog-overlay/30 backdrop-blur-xs"
    >
      <div class="rounded-lg bg-background p-8 text-center">
        <Icon
          class="mx-auto h-16 w-16 text-primary"
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
