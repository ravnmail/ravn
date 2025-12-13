<script lang="ts" setup>
import { computed, reactive, ref, watch, nextTick } from 'vue'
import type { Editor } from '@tiptap/vue-3'
import { BubbleMenu } from '@tiptap/vue-3'
import { useFocus } from '@vueuse/core'

import { useHotkeys, useTiptapStore } from '../hooks'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import AiCompletion from './components/AiCompletion.vue'
// import { useToast } from '@/components/ui/toast/use-toast'
import { Menu } from '@/components/ui/menu'
import { DOMSerializer } from 'prosemirror-model'
import { useAIConversation } from '../hooks/useAIConversation'
import { DEFAULT_SHORTCUTS } from '../extensions/AI/constants'
import type { Props as TippyProps } from 'tippy.js'
import { toast } from 'vue-sonner'

interface Props {
  editor: Editor
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const store = useTiptapStore()
const prompt = ref<string>('')
const cachedPrompt = ref<CachedPrompt | null>(null)
const inputRef = ref<HTMLInputElement | null>(null)
const { focused } = useFocus(inputRef)
const resultContainer = ref<HTMLDivElement | null>(null)
const { t } = useI18n()
const isShaking = ref<boolean>(false)
const tippyInstance = ref<any>(null)
const menuRef = ref()

const { result, status, handleCompletion, resetConversation, stopGeneration } = useAIConversation(props.editor)

interface ShortcutItem {
  label: string
  prompt: string
  icon?: string
  children?: ShortcutItem[]
}

interface MenuItem {
  label: string
  icon?: string
  children?: MenuItem[]

  [key: string]: any
}

interface CachedPrompt {
  context: string
  prompt: string
}

const getSelectionText = (editor: Editor) => {
  const slice = editor.state.selection.content()
  const serializer = DOMSerializer.fromSchema(editor.schema)
  const fragment = serializer.serializeFragment(slice.content)
  const div = document.createElement('div')
  div.appendChild(fragment)

  return div.innerHTML
}

const scrollToBottom = async () => {
  await nextTick()
  if (resultContainer.value) {
    resultContainer.value.scrollTop = resultContainer.value.scrollHeight
  }
}

watch(
  () => result.value,
  () => {
    scrollToBottom()
  }
)

async function handleGenerate() {
  if (!props.editor) {
    toast({
      title: t('editor.AI.error'),
      description: t('editor.AI.editorNotFound'),
      variant: 'destructive',
    })
    return
  }

  try {
    status.value = 'generating'
    const selectionText = getSelectionText(props.editor)

    if (!selectionText.trim()) {
      toast({
        title: t('editor.AI.error'),
        description: t('editor.AI.noSelection'),
        variant: 'destructive',
      })
      return
    }

    await handleCompletion(selectionText, prompt.value)
    cachedPrompt.value = {
      context: selectionText,
      prompt: prompt.value,
    }
    prompt.value = ''
    await nextTick()
    focused.value = true
  } catch (error) {
    toast({
      title: t('editor.AI.error'),
      description: error instanceof Error ? error.message : t('editor.AI.unknownError'),
      variant: 'destructive',
    })
    handleClose()
  }
}

const { bind, unbind } = useHotkeys('esc', () => {
  stopGeneration()
  handleClose()
})

const tippyOptions = reactive<Partial<TippyProps>>({
  maxWidth: 600,
  zIndex: 100,
  appendTo: () => document.body,
  placement: 'bottom-start',
  onShow(instance) {
    tippyInstance.value = instance
    bind()
    setTimeout(() => {
      focused.value = true
    }, 30)
  },
  onHide() {
    unbind()
    handleClose()
  },
  onDestroy() {
    tippyInstance.value = null
    unbind()
  },
})

const shouldShow = computed(() => {
  return store?.state.AIMenu
})

function handleClose() {
  prompt.value = ''
  cachedPrompt.value = null
  resetConversation()
  store!.state.AIMenu = false
}

function handleReGenerate() {
  if (!cachedPrompt.value?.context || !cachedPrompt.value?.prompt) {
    toast({
      title: t('editor.AI.error'),
      description: t('editor.AI.noCachedPrompt'),
      variant: 'destructive',
    })
    return
  }

  try {
    status.value = 'generating'
    resetConversation()
    handleCompletion(cachedPrompt.value.context, cachedPrompt.value.prompt)
      .then(() => {
        scrollToBottom()
      })
      .catch((error) => {
        toast({
          title: t('editor.AI.error'),
          description: error instanceof Error ? error.message : t('editor.AI.regenerateError'),
          variant: 'destructive',
        })
        handleClose()
      })
  } catch (error) {
    toast({
      title: t('editor.AI.error'),
      description: error instanceof Error ? error.message : t('editor.AI.unknownError'),
      variant: 'destructive',
    })
    handleClose()
  }
}

function handleOverlayClick(): void {
  if (status.value === 'init' && prompt.value === '') {
    handleClose()
    return
  }
  isShaking.value = true
  setTimeout(() => {
    isShaking.value = false
  }, 820)
}

function shortcutClick(item: MenuItem) {
  if (!props.editor) {
    toast({
      title: t('editor.AI.error'),
      description: t('editor.AI.editorNotFound'),
      variant: 'destructive',
    })
    return
  }

  try {
    const selectionText = getSelectionText(props.editor)
    const shortcutItem = item as ShortcutItem

    cachedPrompt.value = {
      context: selectionText,
      prompt: shortcutItem.prompt,
    }
    status.value = 'generating'
    handleCompletion(selectionText, shortcutItem.prompt)
      .then(() => {
        scrollToBottom()
        focused.value = true
      })
      .catch((error) => {
        toast({
          title: t('editor.AI.error'),
          description: error instanceof Error ? error.message : t('editor.AI.shortcutError'),
          variant: 'destructive',
        })
        handleClose()
      })
  } catch (error) {
    toast({
      title: t('editor.AI.error'),
      description: error instanceof Error ? error.message : t('editor.AI.unknownError'),
      variant: 'destructive',
    })
    handleClose()
  }
}

const shortcutMenus = computed<ShortcutItem[]>(() => {
  const shortcuts = props.editor?.extensionManager.extensions.find((e) => e.name === 'AI')?.options?.shortcuts
  const mergedShortcuts = [...DEFAULT_SHORTCUTS, ...shortcuts]

  return mergedShortcuts.map((item) => ({
    ...item,
    label: t(item.label),
    children: item.children?.map((child) => ({
      ...child,
      label: t(child.label),
    })),
  }))
})

function handleKey(e) {
  if (status.value === 'init' && shortcutMenus.value.length && !prompt.value) {
    menuRef.value?.handleKeyDown(e)
  }
}
</script>
<template>
  <div
    v-show="shouldShow"
    :style="{
      zIndex: status === 'init' && prompt === '' ? -1 : 98,
    }"
    class="absolute left-0 right-0 top-0 bottom-0"
    @click="handleOverlayClick"
  >
    <BubbleMenu
      v-show="shouldShow"
      :editor="editor"
      :tippy-options="tippyOptions"
      :update-delay="0"
      plugin-key="AIMenu"
    >
      <div
        :class="{ 'shake-animation': isShaking }"
        class="relative w-[450px] z-50"
        @keydown="handleKey"
      >
        <div
          v-show="(status === 'generating' || status === 'completed') && result"
          class="border rounded-md shadow-sm bg-popover border-popover-border"
        >
          <div
            ref="resultContainer"
            class="p-2 block overflow-y-auto max-h-56"
          >
            <div
              :style="{
                padding: 0,
                minHeight: 'auto',
              }"
              class="text-sm"
              v-html="result"
            />
          </div>
        </div>
        <form
          class="relative w-full items-center flex bg-popover mt-3 rounded-md shadow-sm"
          @submit="handleGenerate"
        >
          <div
            v-if="status === 'generating'"
            class="text_loading_animation border border-popover-border h-8 w-full rounded-md px-10 py-1 flex items-center text-sm text-foreground"
          >
            {{ t('composer.AI.generating') }}
          </div>
          <Input
            v-else
            ref="inputRef"
            v-model="prompt"
            :placeholder="t('composer.AI.placeholder')"
            class="px-10 outline-none ring-0 focus-visible:ring-0"
          />
          <span class="absolute start-1 inset-y-0 flex items-center justify-center px-2">
            <Icon
              class="text-ai"
              name="ravn:raven"
            />
          </span>
          <Button
            v-if="status === 'generating'"
            class="absolute end-1 top-1 inset-y-0 flex items-center justify-center size-6"
            size="rounded"
            variant="ai"
            @click="handleClose"
          >
            <Icon
              class="shrink-0"
              name="lucide:x-circle"
            />
          </Button>
          <Button
            v-else
            :disabled="!prompt"
            class="absolute end-1 top-1 inset-y-0 flex items-center justify-center size-6"
            size="rounded"
            variant="ai"
            @click="handleGenerate"
          >
            <Icon
              class="shrink-0"
              name="lucide:arrow-up"
            />
          </Button>
        </form>
        <div
          v-show="status === 'init' && shortcutMenus.length && !prompt"
          class="mt-1 max-w-56"
        >
          <Menu
            ref="menuRef"
            :items="shortcutMenus"
            @item-click="shortcutClick"
          />
        </div>
        <AiCompletion
          v-if="status === 'completed' && prompt === ''"
          :completion="result"
          :editor="editor"
          @close="handleClose"
          @generate="handleReGenerate"
        />
      </div>
    </BubbleMenu>
  </div>
</template>

<style scoped>
@keyframes text-loading {
  0% {
    content: '·';
  }
  33% {
    content: '··';
  }
  66% {
    content: '···';
  }
}

.text_loading_animation::after {
  content: '·';
  margin-left: 8px;
  animation: text-loading 2s infinite;
}

.shake-animation {
  animation: shake 0.82s cubic-bezier(0.36, 0.07, 0.19, 0.97) both;
  transform: translate3d(0, 0, 0);
  backface-visibility: hidden;
  perspective: 1000px;
}

@keyframes shake {
  10%,
  90% {
    transform: translate3d(-1px, 0, 0);
  }

  20%,
  80% {
    transform: translate3d(2px, 0, 0);
  }

  30%,
  50%,
  70% {
    transform: translate3d(-4px, 0, 0);
  }

  40%,
  60% {
    transform: translate3d(4px, 0, 0);
  }
}
</style>
