<script lang="ts" setup>
import type { ButtonViewReturnComponentProps } from '@/types/composer'
import type { TooltipContentProps } from 'reka-ui'
import { useTiptapStore } from '../../../hooks/useStore'
import type { Editor } from '@tiptap/vue-3'
import ActionButton from '@/components/ActionButton.vue'

interface Props {
  editor: Editor
  icon?: string
  title?: string
  tooltip?: string
  disabled?: boolean
  shortcutKeys?: string[]
  customClass?: string
  loading?: boolean
  tooltipOptions?: TooltipContentProps
  color?: string
  action?: ButtonViewReturnComponentProps['action']
  isActive?: ButtonViewReturnComponentProps['isActive']
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  title: undefined,
  tooltip: undefined,
  disabled: false,
  customClass: '',
  color: undefined,
  loading: false,
  shortcutKeys: undefined,
  tooltipOptions: undefined,
  action: undefined,
  isActive: undefined,
})

const store = useTiptapStore()

function handleOpen() {
  const completionsFunc = props.editor.extensionManager.extensions.find(e => e.name === 'AI')?.options?.completions
  if (typeof completionsFunc !== 'function') {
    return
  }
  if (completionsFunc.constructor.name !== 'AsyncFunction') {
    return
  }
  store!.state.AIMenu = true
  props.editor?.commands.focus()
}
</script>

<template>
  <ActionButton
    :action="handleOpen"
    :disabled="disabled"
    :icon="icon"
    :tooltip="tooltip"
    custom-class="text-purple-500"
    title="AI"
  />
</template>
