<script lang="ts" setup>
import type { Editor } from '@tiptap/vue-3'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import ActionButton from '@/components/ActionButton.vue'
import LinkEditBlock from './LinkEditBlock.vue'
import type { ButtonViewReturnComponentProps } from '@/types/composer'

interface Props {
  editor: Editor
  icon?: any
  title?: string
  tooltip?: string
  disabled?: boolean
  shortcutKeys?: string[]
  isActive?: ButtonViewReturnComponentProps['isActive']
  action?: ButtonViewReturnComponentProps['action']
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  title: undefined,
  tooltip: undefined,
  disabled: false,
  shortcutKeys: undefined,
  action: undefined,
  isActive: undefined,
})

function onSetLink(link: string, text?: string, openInNewTab?: boolean) {
  if (props.action) {
    props.action({ link, text, openInNewTab })
  }
}
</script>

<template>
  <Popover>
    <PopoverTrigger :disabled="disabled">
      <ActionButton
        :disabled="disabled"
        :icon="icon"
        :is-active="isActive"
        :shortcut-keys="shortcutKeys"
        :tooltip="tooltip"
      />
    </PopoverTrigger>
    <PopoverContent
      align="start"
      as-child
      class="w-full"
      hide-when-detached
      side="bottom"
    >
      <LinkEditBlock
        :editor="editor"
        @on-set-link="onSetLink"
      />
    </PopoverContent>
  </Popover>
</template>
