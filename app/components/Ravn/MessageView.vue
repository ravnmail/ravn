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
import type { EmailDetail } from '~/types/email'

const { allowImages } = useEmails()
const {
  attachments,
  loadAttachments,
  isLoading: isLoadingAttachments,
  error: attachmentError,
} = useAttachments()
const { formatEmailDate } = useFormatting()
const { getSetting } = useSettings()

const props = withDefaults(
  defineProps<
    EmailDetail & {
      showActions?: boolean
      showAI?: boolean
      autoAnalyze?: boolean
      initialReduced?: boolean
      isFirst?: boolean
    }
  >(),
  {
    showActions: true,
    showAI: true,
    autoAnalyze: false,
    initialReduced: true,
    isFirst: false,
  }
)

const emit = defineEmits<{
  (e: 'reply' | 'reply-all' | 'forward' | 'archive' | 'delete', email: EmailDetail): void
  (e: 'quick-reply', email: EmailDetail, content: string): void
}>()

const handleQuickReply = (content: string) => {
  emit('quick-reply', props as EmailDetail, content)
}

const headerExpanded = ref(false)
const reduced = ref(props.initialReduced)
const iframeRef = ref<HTMLIFrameElement | null>(null)
const imagesBlocked = ref(props.images_blocked)
const showFullContent = ref(false)
const renderMode = ref<'simple' | 'normal'>('simple')
const temporaryRenderMode = ref<'simple' | 'normal' | null>(null)

const { isAnalyzing, analysisError, currentAnalysis, analyzeEmail, parseAnalysisFromCache } =
  useCorvus()
const { updateRead } = useEmails()
const showAnalyzeButton = ref(false)

const effectiveRenderMode = computed(() => {
  return temporaryRenderMode.value || renderMode.value
})

const markAsReadTimout = ref<NodeJS.Timeout | null>(null)

onMounted(async () => {
  try {
    const mode = await getSetting<'simple' | 'normal'>('email.renderMode')
    renderMode.value = mode || 'normal'
  } catch {
    renderMode.value = 'normal'
  }
  if (!props.showAI) return

  markAsReadTimout.value = setTimeout(async () => {
    try {
      if (props.is_read) return
      await updateRead(props.id, true)
    } catch (_: any) {}
  }, 2000)

  const shouldAnalyze = props.category !== 'promotions'
  const cached = parseAnalysisFromCache(props as EmailDetail)
  if (cached) {
    currentAnalysis.value = cached
  } else if (props.autoAnalyze && shouldAnalyze) {
    try {
      await analyzeEmail(props as EmailDetail)
    } catch (_: unknown) {
      showAnalyzeButton.value = true
    }
  } else {
    showAnalyzeButton.value = true
  }
})

onUnmounted(() => {
  if (markAsReadTimout.value) {
    clearTimeout(markAsReadTimout.value)
    markAsReadTimout.value = null
  }
})

const handleAnalyze = async () => {
  try {
    await analyzeEmail(props as EmailDetail)
    showAnalyzeButton.value = false
  } catch (_: unknown) {
    // Error will be shown in analysis component
  }
}

const resolveCidImages = (html: string): string => {
  if (!html) return html
  // Use attachments from useAttachments() — these have content_id + full_path (absolute)
  const inlineAttachments = attachments.value.filter((a) => a.is_inline && a.full_path)
  if (inlineAttachments.length === 0) return html

  const doc = new DOMParser().parseFromString(html, 'text/html')
  const images = doc.querySelectorAll('img')
  images.forEach((img) => {
    const src = img.getAttribute('src') || ''
    if (!src || src.startsWith('data:') || src.startsWith('asset:') || src.startsWith('http'))
      return

    let attachment = undefined

    if (src.startsWith('cid:')) {
      const contentId = src.slice(4).replace(/^<|>$/g, '')
      attachment = inlineAttachments.find(
        (a) => a.content_id === contentId || a.content_id === `<${contentId}>`
      )
    }

    // Fallback: some clients reference inline images by bare filename instead of cid:
    if (!attachment) {
      // Strip any path components — match only on the final filename segment
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

const stripImageSources = (html: string): string => {
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

const hasExternalImages = computed(() => {
  const html = props.body_html || ''
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

const hasQuotedContentAvailable = computed(() => !!props.other_mails)

const getDisplayHtml = computed(() => {
  const html = props.body_html || ''
  const resolved = resolveCidImages(html)
  return imagesBlocked.value ? stripImageSources(resolved) : resolved
})

const hasQuotedContent = computed(() => {
  return hasQuotedContentAvailable.value
})

const toggleRenderMode = () => {
  if (temporaryRenderMode.value) {
    temporaryRenderMode.value = null
  } else {
    temporaryRenderMode.value = renderMode.value === 'simple' ? 'normal' : 'simple'
  }
}

const getQuotedHtml = computed(() => {
  if (!showFullContent.value || !props.other_mails) return ''

  const html = props.other_mails
  const resolved = resolveCidImages(html)
  return imagesBlocked.value ? stripImageSources(resolved) : resolved
})

const hasAnyAttachments = computed(
  () => props.has_attachments || props.attachments?.some((a) => a.is_inline)
)

onMounted(() => {
  if (hasAnyAttachments.value) {
    loadAttachments(props.id)
  }
})

watch(
  () => props.id,
  (newId) => {
    if (hasAnyAttachments.value) {
      loadAttachments(newId)
    }
  }
)

watch(
  () => props.images_blocked,
  (newValue) => {
    imagesBlocked.value = newValue
  }
)

const handleAllowImages = async () => {
  const success = await allowImages(props.id)
  if (success) {
    imagesBlocked.value = false
  }
}

const handleIframeLoad = (event: Event) => {
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

const toggleReduced = () => {
  if (props.isFirst) {
    return
  }
  reduced.value = !reduced.value
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div
      class="items-top flex flex-1"
      @click="toggleReduced()"
    >
      <div class="flex flex-1">
        <RavnAvatar
          v-if="from"
          :email="from.address"
          :name="from.name"
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
                v-bind="from"
              />
              <Icon
                v-if="has_attachments"
                class="ml-1 shrink-0 text-muted"
                name="lucide:paperclip"
              />
            </div>
          </div>
          <div :class="['items-center gap-x-2', headerExpanded ? '' : 'flex flex-wrap']">
            <div
              v-if="to?.length"
              class="flex text-sm"
            >
              <span class="mr-1 text-muted">{{ $t('components.messageView.labels.to') }}: </span>
              <div class="flex flex-wrap">
                <EmailAddress
                  v-for="(a, i) in to"
                  :key="i"
                  :is-last="i === to.length - 1"
                  show-avatar
                  v-bind="a"
                />
              </div>
            </div>
            <div
              v-if="cc?.length"
              class="flex text-sm"
            >
              <span class="mr-1 text-muted">{{ $t('components.messageView.labels.cc') }}: </span>
              <div class="flex flex-wrap">
                <EmailAddress
                  v-for="(a, i) in cc"
                  :key="i"
                  :is-last="i === cc.length - 1"
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
              <span class="text-primary select-auto">{{ subject }}</span>
            </div>
          </div>
        </div>
      </div>
      <div class="flex flex-col items-end justify-between">
        <EmailActionButtons
          v-if="showActions"
          :email="props"
          @archive="emit('archive', $event)"
          @delete="emit('delete', $event)"
          @forward="emit('forward', $event)"
          @reply="emit('reply', $event)"
          @reply-all="emit('reply-all', $event)"
        />
        <div class="ml-auto text-sm">
          {{ formatEmailDate($props, 1, { dateFormat: 'lll' }) }}
        </div>
      </div>
    </div>
    <template v-if="showAI">
      <EmailAIAnalysis
        v-if="currentAnalysis || isAnalyzing"
        :analysis="currentAnalysis"
        :email="props"
        :error="analysisError"
        :is-analyzing="isAnalyzing"
        :reduced="reduced"
        @quick-reply="handleQuickReply"
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
      v-if="labels?.length > 0"
      class="flex flex-wrap gap-1"
    >
      <EmailLabel
        v-for="l in labels"
        :key="l.id"
        v-bind="l"
      />
    </div>
    <template v-if="!reduced">
      <div
        v-if="has_attachments && isLoadingAttachments"
        class="text-muted-foreground flex items-center gap-2 rounded bg-muted/50 p-2 text-sm"
      >
        <Icon
          class="animate-spin"
          name="lucide:loader-2"
        />
        <span>{{ $t('components.messageView.loadingAttachments') }}</span>
      </div>
      <div
        v-else-if="has_attachments && attachmentError"
        class="flex items-center gap-2 rounded bg-destructive/10 p-2 text-sm text-destructive"
      >
        <Icon name="lucide:alert-circle" />
        <span>{{ $t('components.messageView.attachmentError') }}</span>
      </div>
      <AttachmentList
        v-else-if="has_attachments && attachments.length > 0"
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
          v-if="effectiveRenderMode === 'simple' && body_plain"
          class="rounded-xl bg-surface p-3"
        >
          <EmailMarkdown
            :content="body_plain"
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
              v-if="body_plain"
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
