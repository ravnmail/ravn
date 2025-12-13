<script lang="ts" setup>
import { computed, ref } from 'vue'
import type { Editor } from '@tiptap/vue-3'
import { BubbleMenu } from '@tiptap/vue-3'
import LinkEditBlock from '@/lib/editor/extensions/Link/components/LinkEditBlock.vue'
import LinkViewBlock from '@/lib/editor/extensions/Link/components/LinkViewBlock.vue'
import { TextSelection } from '@tiptap/pm/state'

interface Props {
  editor: Editor
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const showEdit = ref(false)
const link = computed(() => {
  const { href: link } = props.editor.getAttributes('link')
  return link
})
const shouldShow = computed<boolean>(() => {
  return props.editor.isActive('link')
})

function onSetLink(url: string, text?: string, openInNewTab?: boolean) {
  props.editor
    .chain()
    .extendMarkRange('link')
    .insertContent({
      type: 'text',
      text: text,
      marks: [
        {
          type: 'link',
          attrs: {
            href: url,
            target: openInNewTab ? '_blank' : '',
          },
        },
      ],
    })
    .setLink({ href: url })
    .focus()
    .run()
  showEdit.value = false
}

function unSetLink() {
  props.editor.chain().extendMarkRange('link').unsetLink().focus().run()
  showEdit.value = false
}

function onClickOutside() {
  const { state, view } = props.editor
  const { tr, selection } = state
  const transaction = tr.setSelection(TextSelection.create(state.doc, selection.from))
  view.dispatch(transaction)
  showEdit.value = false
}
</script>

<template>
  <BubbleMenu
    v-show="shouldShow"
    :editor="editor"
    :tippy-options="{
      popperOptions: {
        modifiers: [{ name: 'flip', enabled: false }],
      },
      appendTo: () => document.body,
      placement: 'bottom-start',
      offset: [-2, 16],
      zIndex: 100,
      onHidden: () => {
        showEdit = false
      },
    }"
    :update-delay="0"
  >
    <LinkEditBlock
      v-if="showEdit"
      :editor="editor"
      @on-set-link="onSetLink"
      @on-click-outside="onClickOutside"
    />
    <LinkViewBlock
      v-else
      :editor="editor"
      :link="link"
      @clear="unSetLink"
      @edit="showEdit = true"
    />
  </BubbleMenu>
</template>
