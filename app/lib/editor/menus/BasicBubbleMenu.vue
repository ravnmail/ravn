<script lang="ts" setup>
import { computed, reactive, unref } from 'vue'
import type { NodeSelection } from '@tiptap/pm/state'
import { TextSelection } from '@tiptap/pm/state'
import { Separator } from '~/components/ui/separator'
import type { Editor, Extension } from '@tiptap/vue-3'
import { BubbleMenu } from '@tiptap/vue-3'
import type { MailKitOptions } from '~/lib/editor/extensions/Mailkit'
import type { BubbleTypeMenu } from './BasicBubble'
import { useTiptapStore } from '~/lib/editor/hooks'

interface Props {
  editor: Editor
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const store = useTiptapStore()

const { t } = useI18n()
const tippyOptions = reactive<Record<string, unknown>>({
  maxWidth: 'auto',
  zIndex: 100,
  appendTo: () => document.body,
  moveTransition: 'transform 0.15s ease-out',
})

const nodeType = computed(() => {
  const selection = props.editor.state.selection as NodeSelection
  const isLink = props.editor.isActive('link')
  const isVideo = selection.node?.type.name === 'video'
  const isText = selection instanceof TextSelection
  if (isLink) return 'link'
  if (isVideo) return 'video'
  if (isText) return 'text'
  return undefined
})

const nodeMenus = computed(() => {
  const { extensions = [] } = props.editor.extensionManager
  const find = extensions.find(k => k.name === 'mail-kit') as Extension<MailKitOptions>
  if (!find) return {}

  const { button } = find.options?.bubble ?? {}

  if (!button) return {}

  const _button: BubbleTypeMenu = button({
    editor: props.editor,
    extension: find,
    t: unref(t),
  })

  return _button
})

const items = computed(() => {
  if (!nodeType.value) return []
  return unref(nodeMenus)?.[nodeType.value] ?? []
})
</script>
<template>
  <BubbleMenu
v-show="items.length && !store?.state.AIMenu"
:editor="editor"
:tippy-options="tippyOptions">
    <div
      class="border border-neutral-200 dark:border-neutral-800 px-3 py-2 transition-all select-none pointer-events-auto shadow-sm rounded-sm bg-background w-auto max-w-[calc(-68px_+_100vw)] overflow-x-auto"
    >
      <div class="flex items-center flex-nowrap whitespace-nowrap h-[26px] justify-start relative gap-0.5">
        <template
v-for="(item, key) in items"
:key="key">
          <!-- Divider -->
          <Separator
v-if="item.type === 'divider'"
class="mx-1 me-1 h-[16px]"
orientation="vertical" />
          <!-- Buttons -->
          <component
            :is="item.component"
            v-else
            :disabled="disabled || item.componentProps?.disabled"
            :editor="editor"
            v-bind="item.componentProps"
          >
            <template
v-for="(element, slotName, i) in item.componentSlots"
:key="i"
#[`${slotName}`]="values">
              <component
:is="element"
v-bind="values?.props" />
            </template>
          </component>
        </template>
      </div>
    </div>
  </BubbleMenu>
</template>
