<script lang="ts" setup>

import { invoke } from '@tauri-apps/api/core'
import type { EmailDetail } from '~/types/email'
import EmailAddress from '~/components/Ravn/EmailAddress.vue'
import AttachmentList from '~/components/Ravn/AttachmentList.vue'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import EmailActionButtons from '~/components/Ravn/EmailActionButtons.vue'
import EmailAIAnalysis from '~/components/Ravn/EmailAIAnalysis.vue'
import EmailMarkdown from '~/components/Ravn/EmailMarkdown.vue'
import { Button } from '~/components/ui/button'

const { allowImages, allowAll } = useEmails()
const { attachments, loadAttachments, isLoading: isLoadingAttachments, error: attachmentError } = useAttachments()
const { formatEmailDate } = useFormatting()
const { settings, getSetting } = useSettings()

const props = withDefaults(defineProps<EmailDetail & {
  showActions?: boolean
  showAI?: boolean
  autoAnalyze?: boolean
  reduced?: boolean
}>(), {
  showActions: true,
  showAI: true,
  autoAnalyze: false,
  reduced: false
})

const emit = defineEmits<{
  (e: 'reply' | 'reply-all' | 'forward' | 'archive' | 'delete', email: EmailDetail): void
  (e: 'quick-reply', email: EmailDetail, content: string): void
}>()

const handleQuickReply = (content: string) => {
  emit('quick-reply', props as EmailDetail, content)
}

const expanded = ref(false)
const iframeRef = ref<HTMLIFrameElement | null>(null)
const imagesBlocked = ref(props.images_blocked)
const trackingBlocked = ref(props.tracking_blocked)
const showFullContent = ref(false)
const renderMode = ref<'simple' | 'normal'>('simple')
const temporaryRenderMode = ref<'simple' | 'normal' | null>(null)

const {
  isAnalyzing,
  analysisError,
  currentAnalysis,
  analyzeEmail,
  parseAnalysisFromCache
} = useCorvus()
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
    } catch (_: any) {
    }
  }, 2000)

  const shouldAnalyze = props.category === 'personal'
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

const stripImageSources = (html: string): string => {
  const doc = new DOMParser().parseFromString(html, 'text/html')

  const images = doc.querySelectorAll('img')
  images.forEach((img) => {
    const src = img.getAttribute('src') || ''
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
  return imagesBlocked.value ? stripImageSources(html) : html
})

const hasQuotedContent = computed(() => {
  if (!props.reduced) return false
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
  return imagesBlocked.value ? stripImageSources(html) : html
})

onMounted(() => {
  if (props.has_attachments) {
    loadAttachments(props.id)
  }
})

watch(
  () => props.id,
  (newId) => {
    if (props.has_attachments) {
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

watch(
  () => props.tracking_blocked,
  (newValue) => {
    trackingBlocked.value = newValue
  }
)

const handleAllowImages = async () => {
  const success = await allowImages(props.id)
  if (success) {
    imagesBlocked.value = false
  }
}

const handleAllowAll = async () => {
  const success = await allowAll(props.id)
  if (success) {
    imagesBlocked.value = false
    trackingBlocked.value = false
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

</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="flex flex-1 items-top">
      <div class="flex flex-1">
        <RavnAvatar
          v-if="from"
          :account-id="account_id"
          :email="from.address"
          :name="from.name"
          class="shrink-0 mr-4"
          size="lg"
        />
        <div class="grow">
          <div class="flex items-center">
            <div class="flex gap-1 items-center">
          <span
            v-if="expanded"
            class="text-muted"
          >{{ $t('components.messageView.labels.from') }}: </span>
              <EmailAddress
                :account-id="account_id"
                :show-avatar="expanded"
                class="font-bold"
                is-last
                v-bind="from"
              />
              <Icon
                v-if="has_attachments"
                class="shrink-0 text-muted ml-1"
                name="lucide:paperclip"
              />
            </div>
          </div>
          <div :class="['gap-x-2 items-center', expanded ? '' : 'flex flex-wrap']">
            <div
              v-if="to?.length"
              class="text-sm flex"
            >
              <span class="text-muted mr-1">{{ $t('components.messageView.labels.to') }}: </span>
              <div class="flex flex-wrap">
                <EmailAddress
                  v-for="(a, i) in to"
                  :key="i"
                  :account-id="account_id"
                  :is-last="i === to.length - 1"
                  show-avatar
                  v-bind="a"
                />
              </div>
            </div>
            <div
              v-if="cc?.length"
              class="text-sm flex"
            >
              <span class="text-muted mr-1">{{ $t('components.messageView.labels.cc') }}: </span>
              <div class="flex flex-wrap">
                <EmailAddress
                  v-for="(a, i) in cc"
                  :key="i"
                  :account-id="account_id"
                  :is-last="i === cc.length - 1"
                  show-avatar
                  v-bind="a"
                />
              </div>
            </div>
            <Icon
              v-if="!expanded"
              class="text-muted"
              name="lucide:chevron-right"
              @click="expanded = !expanded"
            />
            <div
              v-if="expanded"
              class="flex space-x-1 text-sm"
            >
              <span
                class="text-muted"
              >{{ $t('components.messageView.labels.subject') }}: </span>
              <span class="select-auto text-primary">{{ subject }}</span>
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
        <div class="ml-auto text-sm ">
          {{ formatEmailDate($props, 1, { dateFormat: 'lll' }) }}
        </div>
      </div>
    </div>
    <!-- AI Analysis -->
    <template v-if="showAI">
      <EmailAIAnalysis
        v-if="currentAnalysis || isAnalyzing"
        :analysis="currentAnalysis"
        :email="props"
        :error="analysisError"
        :is-analyzing="isAnalyzing"
        @quick-reply="handleQuickReply"
      />
      <div
        v-else-if="showAnalyzeButton"
      >
        <Button
          size="sm"
          variant="outline"
          @click="handleAnalyze"
        >
          <Icon name="ravn:raven"/>
          <span>{{ $t('components.messageView.actions.analyzeWithAI') }}</span>
        </Button>
      </div>
    </template>
    <div
      v-if="labels?.length > 0"
      class="flex gap-1 flex-wrap"
    >
      <EmailLabel
        v-for="l in labels"
        :key="l.id"
        v-bind="l"
      />
    </div>
    <div
      v-if="has_attachments && isLoadingAttachments"
      class="flex items-center gap-2 text-sm text-muted-foreground p-2 bg-muted/50 rounded"
    >
      <Icon
        class="animate-spin"
        name="lucide:loader-2"
      />
      <span>{{ $t('components.messageView.loadingAttachments') }}</span>
    </div>
    <div
      v-else-if="has_attachments && attachmentError"
      class="flex items-center gap-2 text-sm text-destructive p-2 bg-destructive/10 rounded"
    >
      <Icon name="lucide:alert-circle"/>
      <span>{{ $t('components.messageView.attachmentError') }}</span>
    </div>
    <AttachmentList
      v-else-if="has_attachments && attachments.length > 0"
      :attachments="attachments.filter(a => !a.is_inline)"
    />
    <div
      v-if="imagesBlocked && hasExternalImages"
      class="flex items-center justify-between bg-surface p-2 border-border border rounded text-xs"
    >
      <div class="flex items-center gap-2">
        <Icon
          class="shrink-0"
          name="lucide:image-off"
        />
        <span>{{ $t('components.messageView.imagesBlocked') }}</span>
      </div>
      <Button
        size="xs"
        variant="outline"
        @click="handleAllowImages"
      >{{ $t('components.messageView.actions.showImages') }}
      </Button>
    </div>
    <div class="flex flex-col gap-1">
      <div
        v-if="effectiveRenderMode === 'simple' && body_plain"
        class="bg-surface rounded p-4"
      >
        <EmailMarkdown
          :content="body_plain"
          :images-blocked="imagesBlocked"
        />
      </div>
      <div
        v-else
        class="bg-gray-50 text-gray-950 rounded overflow-clip"
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
        v-if="reduced && showFullContent && getQuotedHtml"
        class="bg-gray-100 text-gray-950 rounded overflow-clip p-2 border-l-4 border-accent ml-6"
      >
        <iframe
          :srcdoc="getQuotedHtml"
          class="w-full border-0"
          loading="lazy"
          sandbox="allow-same-origin allow-scripts"
          @load="handleIframeLoad"
        />
      </div>
      <div class="flex justify-center gap-2">
        <Button
          v-if="reduced && hasQuotedContent"
          class="text-muted-foreground hover:text-primary"
          size="sm"
          variant="ghost"
          @click="showFullContent = !showFullContent"
        >
          <Icon
            class="mr-1"
            name="lucide:chevron-down"
          />
          <span>{{ $t('components.messageView.actions.showMore') }}</span>
        </Button>
        <Button
          v-if="body_plain"
          class="text-muted-foreground hover:text-primary"
          size="sm"
          variant="ghost"
          @click="toggleRenderMode"
        >
          <Icon
            :name="effectiveRenderMode === 'simple' ? 'lucide:code' : 'lucide:file-text'"
            class="mr-1"
          />
          <span>{{ effectiveRenderMode === 'simple' ? $t('components.messageView.actions.showHTML') : $t('components.messageView.actions.showSimple') }}</span>
        </Button>
      </div>
    </div>
  </div>
</template>