<script lang="ts" setup>
import { convertFileSrc, invoke } from '@tauri-apps/api/core'

import AttachmentList from '~/components/Ravn/AttachmentList.vue'
import EmailActionButtons from '~/components/Ravn/EmailActionButtons.vue'
import EmailAddress from '~/components/Ravn/EmailAddress.vue'
import EmailAIAnalysis from '~/components/Ravn/EmailAIAnalysis.vue'
import EmailMarkdown from '~/components/Ravn/EmailMarkdown.vue'
import { Button } from '~/components/ui/button'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import { SimpleTooltip } from '~/components/ui/tooltip'
import type { EmailAnalysis, EmailDetail } from '~/types/email'

const props = withDefaults(
  defineProps<
    EmailDetail & {
      showActions?: boolean
      showAI?: boolean
      autoAnalyze?: boolean
      initialReduced?: boolean
      isFirst?: boolean
      isHighlighted?: boolean
    }
  >(),
  {
    showActions: true,
    showAI: true,
    autoAnalyze: false,
    initialReduced: true,
    isFirst: false,
    isHighlighted: false,
  }
)

const emit = defineEmits<{
  (e: 'reply' | 'reply-all' | 'forward' | 'archive' | 'delete', email: EmailDetail): void
  (e: 'quick-reply', email: EmailDetail, content: string): void
}>()

const { allowImages, updateRead } = useEmails()
const {
  attachments,
  loadAttachments,
  isLoading: isLoadingAttachments,
  error: attachmentError,
} = useAttachments()
const { formatEmailDate } = useFormatting()
const { getSetting } = useSettings()
const { analyzeEmail, reanalyzeEmail } = useCorvus()

const headerExpanded = ref(false)
const reduced = ref(props.initialReduced)
const iframeRef = ref<HTMLIFrameElement | null>(null)
const imagesBlocked = ref(props.images_blocked)
const showFullContent = ref(false)
const renderMode = ref<'simple' | 'normal'>('simple')
const temporaryRenderMode = ref<'simple' | 'normal' | null>(null)

const localEmail = reactive<EmailDetail>({
  ...props,
  attachments: [...(props.attachments || [])],
  labels: [...(props.labels || [])],
  to: [...(props.to || [])],
  cc: [...(props.cc || [])],
  bcc: [...(props.bcc || [])],
})

const isAnalyzing = ref(false)
const analysisError = ref<string | null>(null)
const showAnalyzeButton = ref(false)
const markAsReadTimout = ref<NodeJS.Timeout | null>(null)
const autoAnalysisAttemptedForEmailId = ref<string | null>(null)

const effectiveRenderMode = computed(() => temporaryRenderMode.value || renderMode.value)

function parseAnalysis(value: unknown): EmailAnalysis | null {
  if (!value) return null

  try {
    const parsed =
      typeof value === 'string' ? (JSON.parse(value) as EmailAnalysis) : (value as EmailAnalysis)

    if (
      parsed &&
      typeof parsed.gist === 'string' &&
      Array.isArray(parsed.responses) &&
      parsed.responses.every(
        (response) =>
          response && typeof response.title === 'string' && typeof response.content === 'string'
      )
    ) {
      return parsed
    }

    return null
  } catch (error) {
    console.error('Failed to parse ai_cache:', error)
    return null
  }
}

const currentAnalysis = computed(() => parseAnalysis(localEmail.ai_cache))
const hasAnyAttachments = computed(
  () => localEmail.has_attachments || localEmail.attachments?.some((a) => a.is_inline)
)
const hasQuotedContentAvailable = computed(() => !!localEmail.other_mails)
const hasQuotedContent = computed(() => hasQuotedContentAvailable.value)

const hasExternalImages = computed(() => {
  const html = localEmail.body_html || ''
  if (!html) return false

  const doc = new DOMParser().parseFromString(html, 'text/html')
  const images = doc.querySelectorAll('img')

  for (const img of images) {
    const src = img.getAttribute('src') || ''
    if (src && !src.startsWith('data:') && !src.startsWith('/') && !src.startsWith('cid:')) {
      return true
    }
  }

  return false
})

function syncLocalEmailFromProps() {
  localEmail.id = props.id
  localEmail.account_id = props.account_id
  localEmail.folder_id = props.folder_id
  localEmail.message_id = props.message_id
  localEmail.conversation_id = props.conversation_id
  localEmail.remote_id = props.remote_id
  localEmail.from = props.from
  localEmail.to = [...(props.to || [])]
  localEmail.cc = [...(props.cc || [])]
  localEmail.bcc = [...(props.bcc || [])]
  localEmail.reply_to = props.reply_to
  localEmail.subject = props.subject
  localEmail.snippet = props.snippet
  localEmail.body_plain = props.body_plain
  localEmail.body_html = props.body_html
  localEmail.other_mails = props.other_mails
  localEmail.ai_cache = props.ai_cache
  localEmail.headers = props.headers
  localEmail.size = props.size
  localEmail.received_at = props.received_at
  localEmail.sent_at = props.sent_at
  localEmail.scheduled_send_at = props.scheduled_send_at
  localEmail.remind_at = props.remind_at
  localEmail.notified_at = props.notified_at
  localEmail.is_read = props.is_read
  localEmail.is_flagged = props.is_flagged
  localEmail.is_draft = props.is_draft
  localEmail.has_attachments = props.has_attachments
  localEmail.sync_status = props.sync_status
  localEmail.body_fetch_attempts = props.body_fetch_attempts
  localEmail.last_body_fetch_attempt = props.last_body_fetch_attempt
  localEmail.tracking_blocked = props.tracking_blocked
  localEmail.images_blocked = props.images_blocked
  localEmail.created_at = props.created_at
  localEmail.updated_at = props.updated_at
  localEmail.labels = [...(props.labels || [])]
  localEmail.attachments = [...(props.attachments || [])]
  localEmail.category = props.category
}

function applyAnalysis(analysis: EmailAnalysis | null) {
  localEmail.ai_cache = analysis ? JSON.stringify(analysis) : undefined
  analysisError.value = null
  showAnalyzeButton.value = !analysis
}

async function runAnalysis(mode: 'normal' | 'force') {
  const emailId = localEmail.id
  autoAnalysisAttemptedForEmailId.value = emailId

  if (mode === 'force') {
    localEmail.ai_cache = undefined
  }

  try {
    isAnalyzing.value = true
    analysisError.value = null

    const analysis =
      mode === 'force' ? await reanalyzeEmail(localEmail) : await analyzeEmail(localEmail)

    applyAnalysis(analysis)
  } catch (error) {
    analysisError.value = error instanceof Error ? error.message : 'Failed to analyze email'
    showAnalyzeButton.value = true
  } finally {
    isAnalyzing.value = false
  }
}

async function maybeAnalyzeEmail() {
  if (!props.showAI) return

  const shouldAnalyze = localEmail.category !== 'promotions'

  if (currentAnalysis.value) {
    showAnalyzeButton.value = false
    autoAnalysisAttemptedForEmailId.value = localEmail.id
    return
  }

  if (!props.autoAnalyze || !shouldAnalyze) {
    showAnalyzeButton.value = true
    return
  }

  if (isAnalyzing.value || autoAnalysisAttemptedForEmailId.value === localEmail.id) {
    return
  }

  await runAnalysis('normal')
}

async function handleAnalyze() {
  await runAnalysis('normal')
}

async function handleReanalyze() {
  await runAnalysis('force')
}

function handleQuickReply(content: string) {
  emit('reply-all', localEmail)
  emit('quick-reply', localEmail, content)
}

function resolveCidImages(html: string): string {
  if (!html) return html

  const inlineAttachments = attachments.value.filter((a) => a.is_inline && a.full_path)
  if (inlineAttachments.length === 0) return html

  const doc = new DOMParser().parseFromString(html, 'text/html')
  const images = doc.querySelectorAll('img')

  images.forEach((img) => {
    const src = img.getAttribute('src') || ''
    if (!src || src.startsWith('data:') || src.startsWith('asset:') || src.startsWith('http')) {
      return
    }

    let attachment

    if (src.startsWith('cid:')) {
      const contentId = src.slice(4).replace(/^<|>$/g, '')
      attachment = inlineAttachments.find(
        (a) => a.content_id === contentId || a.content_id === `<${contentId}>`
      )
    }

    if (!attachment) {
      const srcFilename = src.split('/').pop()?.split('?')[0] ?? ''
      if (srcFilename) {
        attachment = inlineAttachments.find(
          (a) => a.filename.toLowerCase() === srcFilename.toLowerCase()
        )
      }
    }

    if (attachment?.full_path) {
      img.setAttribute('src', convertFileSrc(attachment.full_path))
      img.setAttribute('data-cid-resolved', 'true')
    }
  })

  return doc.documentElement.innerHTML
}

function stripImageSources(html: string): string {
  const doc = new DOMParser().parseFromString(html, 'text/html')

  const images = doc.querySelectorAll('img')
  images.forEach((img) => {
    const src = img.getAttribute('src') || ''
    const isCidResolved = img.getAttribute('data-cid-resolved') === 'true'
    if (isCidResolved) return
    if (src && !src.startsWith('data:') && !src.startsWith('/')) {
      img.removeAttribute('src')
    }
  })

  const sources = doc.querySelectorAll('source')
  sources.forEach((source) => {
    source.removeAttribute('src')
  })

  const scripts = doc.querySelectorAll('script')
  scripts.forEach((script) => {
    script.remove()
  })

  return doc.documentElement.innerHTML
}

const getDisplayHtml = computed(() => {
  const html = localEmail.body_html || ''
  const resolved = resolveCidImages(html)
  return imagesBlocked.value ? stripImageSources(resolved) : resolved
})

const getQuotedHtml = computed(() => {
  if (!showFullContent.value || !localEmail.other_mails) return ''

  const html = localEmail.other_mails
  const resolved = resolveCidImages(html)
  return imagesBlocked.value ? stripImageSources(resolved) : resolved
})

async function handleAllowImages() {
  const success = await allowImages(localEmail.id)
  if (success) {
    imagesBlocked.value = false
  }
}

function handleIframeLoad(event: Event) {
  const iframe = event.target as HTMLIFrameElement
  try {
    const doc = iframe.contentDocument || iframe.contentWindow?.document
    if (!doc) return

    const h = Math.max(doc.documentElement.scrollHeight, doc.body.scrollHeight)
    iframe.style.height = `${h}px`
    iframe.style.overflow = 'hidden'

    const links = doc.querySelectorAll('a')
    links.forEach((link) => {
      link.addEventListener('click', (e) => {
        e.preventDefault()
        const url = link.getAttribute('href')
        if (url) {
          invoke('open_external_url', { url }).catch((err) => {
            console.error('Failed to open external URL:', err)
          })
        }
      })
    })
  } catch (e) {
    console.error('Failed to handle iframe load:', e)
  }
}

function toggleRenderMode() {
  if (temporaryRenderMode.value) {
    temporaryRenderMode.value = null
  } else {
    temporaryRenderMode.value = renderMode.value === 'simple' ? 'normal' : 'simple'
  }
}

function toggleReduced() {
  if (props.isFirst) return
  reduced.value = !reduced.value
}

watch(
  () => props,
  () => {
    syncLocalEmailFromProps()
    imagesBlocked.value = props.images_blocked
  },
  { deep: true }
)

watch(
  () => props.id,
  async (newId, oldId) => {
    if (!newId || newId === oldId) return

    autoAnalysisAttemptedForEmailId.value = null
    syncLocalEmailFromProps()
    analysisError.value = null
    showAnalyzeButton.value = false

    if (hasAnyAttachments.value) {
      loadAttachments(newId)
    }

    await maybeAnalyzeEmail()
  }
)

watch(
  () => props.images_blocked,
  (newValue) => {
    imagesBlocked.value = newValue
  }
)

onMounted(async () => {
  syncLocalEmailFromProps()

  try {
    const mode = await getSetting<'simple' | 'normal'>('email.renderMode')
    renderMode.value = mode || 'normal'
  } catch {
    renderMode.value = 'normal'
  }

  markAsReadTimout.value = setTimeout(async () => {
    try {
      if (localEmail.is_read) return
      await updateRead(localEmail.id, true)
    } catch (_: any) {}
  }, 2000)

  if (hasAnyAttachments.value) {
    loadAttachments(localEmail.id)
  }

  await maybeAnalyzeEmail()
})

onUnmounted(() => {
  if (markAsReadTimout.value) {
    clearTimeout(markAsReadTimout.value)
    markAsReadTimout.value = null
  }
})
</script>

<template>
  <div
    :class="[
      'flex flex-col gap-3 rounded-2xl transition-colors',
      isHighlighted ? 'bg-accent/5 p-3 ring-1 ring-accent/30' : '',
    ]"
  >
    <div
      class="items-top flex flex-1"
      @click="toggleReduced()"
    >
      <div class="flex flex-1">
        <RavnAvatar
          v-if="localEmail.from"
          :email="localEmail.from.address"
          :name="localEmail.from.name"
          class="mr-4 shrink-0"
          size="lg"
        />
        <div class="grow">
          <div class="flex items-center">
            <div class="flex items-center gap-1">
              <span
                v-if="headerExpanded"
                class="text-muted"
                >{{ $t('components.messageView.labels.from') }}:
              </span>
              <EmailAddress
                :show-avatar="headerExpanded"
                class="font-bold"
                is-last
                v-bind="localEmail.from"
              />
              <Icon
                v-if="localEmail.has_attachments"
                class="ml-1 shrink-0 text-muted"
                name="lucide:paperclip"
              />
            </div>
          </div>
          <div :class="['items-center gap-x-2', headerExpanded ? '' : 'flex flex-wrap']">
            <div
              v-if="localEmail.to?.length"
              class="flex text-sm"
            >
              <span class="mr-1 text-muted">{{ $t('components.messageView.labels.to') }}: </span>
              <div class="flex flex-wrap">
                <EmailAddress
                  v-for="(a, i) in localEmail.to"
                  :key="i"
                  :is-last="i === localEmail.to.length - 1"
                  show-avatar
                  v-bind="a"
                />
              </div>
            </div>
            <div
              v-if="localEmail.cc?.length"
              class="flex text-sm"
            >
              <span class="mr-1 text-muted">{{ $t('components.messageView.labels.cc') }}: </span>
              <div class="flex flex-wrap">
                <EmailAddress
                  v-for="(a, i) in localEmail.cc"
                  :key="i"
                  :is-last="i === localEmail.cc.length - 1"
                  show-avatar
                  v-bind="a"
                />
              </div>
            </div>
            <Icon
              v-if="!headerExpanded"
              class="text-muted"
              name="lucide:chevron-right"
              @click="headerExpanded = !headerExpanded"
            />
            <div
              v-if="headerExpanded"
              class="flex space-x-1 text-sm"
            >
              <span class="text-muted">{{ $t('components.messageView.labels.subject') }}: </span>
              <span class="text-primary select-auto">{{ localEmail.subject }}</span>
            </div>
          </div>
        </div>
      </div>
      <div class="flex flex-col items-end justify-between">
        <EmailActionButtons
          v-if="showActions"
          :email="localEmail"
          @archive="emit('archive', $event)"
          @delete="emit('delete', $event)"
          @forward="emit('forward', $event)"
          @reply="emit('reply', $event)"
          @reply-all="emit('reply-all', $event)"
        />
        <div class="ml-auto text-sm">
          {{ formatEmailDate(localEmail, 1, { dateFormat: 'lll' }) }}
        </div>
      </div>
    </div>

    <template v-if="showAI">
      <EmailAIAnalysis
        v-if="currentAnalysis || isAnalyzing || analysisError"
        :analysis="currentAnalysis"
        :email="localEmail"
        :error="analysisError"
        :is-analyzing="isAnalyzing"
        :reduced="reduced"
        @quick-reply="handleQuickReply"
        @regenerate="handleReanalyze"
      />
      <div v-else-if="showAnalyzeButton && !reduced">
        <Button
          size="sm"
          variant="outline"
          @click="handleAnalyze"
        >
          <Icon name="ravn:raven" />
          <span>{{ $t('components.messageView.actions.analyzeWithAI') }}</span>
        </Button>
      </div>
    </template>

    <div
      v-if="localEmail.labels?.length > 0"
      class="flex flex-wrap gap-1"
    >
      <EmailLabel
        v-for="l in localEmail.labels"
        :key="l.id"
        v-bind="l"
      />
    </div>

    <template v-if="!reduced">
      <div
        v-if="hasAnyAttachments && isLoadingAttachments"
        class="text-muted-foreground flex items-center gap-2 rounded bg-muted/50 p-2 text-sm"
      >
        <Icon
          class="animate-spin"
          name="lucide:loader-2"
        />
        <span>{{ $t('components.messageView.loadingAttachments') }}</span>
      </div>

      <div
        v-else-if="hasAnyAttachments && attachmentError"
        class="flex items-center gap-2 rounded bg-destructive/10 p-2 text-sm text-destructive"
      >
        <Icon name="lucide:alert-circle" />
        <span>{{ $t('components.messageView.attachmentError') }}</span>
      </div>

      <AttachmentList
        v-else-if="hasAnyAttachments && attachments.length > 0"
        :attachments="attachments.filter((a) => !a.is_inline)"
      />

      <div
        v-if="imagesBlocked && hasExternalImages"
        class="flex items-center justify-between rounded border-border bg-surface p-1 text-xs"
      >
        <div class="flex items-center gap-2 pl-1">
          <Icon
            class="shrink-0"
            name="lucide:image-off"
          />
          <span>{{ $t('components.messageView.imagesBlocked') }}</span>
        </div>
        <Button
          size="xs"
          variant="ghost"
          @click="handleAllowImages"
          >{{ $t('components.messageView.actions.showImages') }}
        </Button>
      </div>

      <div class="relative flex flex-col">
        <div
          v-if="effectiveRenderMode === 'simple' && localEmail.body_plain"
          class="rounded-xl bg-surface p-3"
        >
          <EmailMarkdown
            :content="localEmail.body_plain"
            :images-blocked="imagesBlocked"
            :inline-attachments="attachments.filter((a) => a.is_inline && !!a.full_path)"
          />
        </div>

        <div
          v-else
          class="overflow-clip rounded-xl bg-gray-50 p-2 text-gray-950"
        >
          <iframe
            ref="iframeRef"
            :srcdoc="getDisplayHtml"
            class="w-full border-0"
            loading="lazy"
            sandbox="allow-same-origin allow-scripts"
            @load="handleIframeLoad"
          />
        </div>

        <div
          v-if="showFullContent && getQuotedHtml"
          class="mt-2 ml-12 overflow-clip rounded border-l-4 border-accent bg-gray-50 p-2"
        >
          <iframe
            :srcdoc="getQuotedHtml"
            class="w-full border-0"
            loading="lazy"
            sandbox="allow-same-origin allow-scripts"
            @load="handleIframeLoad"
          />
        </div>

        <div class="absolute top-2 right-2 flex flex-col justify-center gap-1">
          <SimpleTooltip
            :tooltip-markdown="
              effectiveRenderMode === 'simple'
                ? $t('components.messageView.actions.showHTML')
                : $t('components.messageView.actions.showSimple')
            "
          >
            <Button
              v-if="localEmail.body_plain"
              size="icon"
              variant="ghost"
              @click="toggleRenderMode"
            >
              <Icon :name="effectiveRenderMode === 'simple' ? 'lucide:code' : 'lucide:file-text'" />
            </Button>
          </SimpleTooltip>

          <SimpleTooltip
            :tooltip-markdown="
              showFullContent
                ? $t('components.messageView.actions.showLess')
                : $t('components.messageView.actions.showMore')
            "
          >
            <Button
              v-if="hasQuotedContent"
              size="icon"
              variant="ghost"
              @click="showFullContent = !showFullContent"
            >
              <Icon :name="showFullContent ? 'lucide:fold-vertical' : 'lucide:unfold-vertical'" />
            </Button>
          </SimpleTooltip>
        </div>
      </div>
    </template>
  </div>
</template>
