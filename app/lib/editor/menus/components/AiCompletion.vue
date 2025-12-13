<script lang="ts" setup>
import { Editor } from '@tiptap/core'
import { MenuItem } from '@/components/ui/menu'

const props = defineProps({
  editor: {
    type: Editor,
    required: true,
  },
  completion: {
    type: String,
    required: true,
  },
})
const { t } = useI18n()
const emits = defineEmits(['generate', 'close'])

function handleReplace() {
  const selection = props.editor.view.state.selection
  props.editor
    .chain()
    .focus()
    .deleteRange({
      from: selection.from,
      to: selection.to,
    })
    .insertContent(props.completion, {
      parseOptions: {
        preserveWhitespace: false,
      },
      updateSelection: true,
    })
    .run()
}

function handleInsert() {
  const { to } = props.editor.view.state.selection
  if (to) {
    props.editor
      .chain()
      .focus()
      .insertContentAt(to + 1, props.completion)
      .run()
  }
}

function handleGenerate() {
  emits('generate')
}

function handleClose() {
  emits('close')
}
</script>

<template>
  <div class="bg-popover mt-1 flex-wrap border border-popover-border rounded p-1 max-w-60">
    <div class="flex flex-col gap-1">
      <MenuItem @click="handleReplace">
        <Icon name="lucide:replace"/>
        {{ t('composer.AI.replace') }}
      </MenuItem>
      <MenuItem @click="handleInsert">
        <Icon name="lucide:text-quote"/>
        {{ t('composer.AI.insert') }}
      </MenuItem>
      <MenuItem @click="handleGenerate">
        <Icon name="lucide:refresh-cw"/>
        {{ t('composer.AI.regenerate') }}
      </MenuItem>
      <MenuItem @click="handleClose">
        <Icon name="lucide:trash-2"/>
        {{ t('composer.AI.close') }}
      </MenuItem>
    </div>
  </div>
</template>
