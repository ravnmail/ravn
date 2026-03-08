<script lang="ts" setup>
import { pointerOutsideOfPreview } from '@atlaskit/pragmatic-drag-and-drop/element/pointer-outside-of-preview'
import { setCustomNativeDragPreview } from '@atlaskit/pragmatic-drag-and-drop/element/set-custom-native-drag-preview'

import EmailItem from '~/components/Ravn/EmailItem.vue'
import { useDraggable } from '~/composables/useDragAndDrop'
import type { EmailListItem } from '~/types/email'

const props = defineProps<{
  email: EmailListItem
  swimlaneId: string
  excludeLabels?: string[]
  isSelected?: boolean
}>()

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void
}>()

const emailRef = ref<HTMLElement | null>(null)

const getDragData = () => ({
  type: 'email' as const,
  id: props.email.id,
  accountId: props.email.account_id,
  folderId: props.email.folder_id,
  fromSwimlaneId: props.swimlaneId,
})

const { isDragging } = useDraggable(emailRef, getDragData, {
  onGenerateDragPreview: ({ nativeSetDragImage }) => {
    setCustomNativeDragPreview({
      nativeSetDragImage,
      getOffset: pointerOutsideOfPreview({ x: '16px', y: '8px' }),
      render({ container }) {
        const el = document.createElement('div')
        el.style.cssText = [
          'display:flex',
          'align-items:center',
          'gap:6px',
          'padding:6px 10px',
          'border-radius:8px',
          'font-size:13px',
          'font-weight:500',
          'max-width:220px',
          'white-space:nowrap',
          'overflow:hidden',
          'text-overflow:ellipsis',
          'box-shadow:0 4px 16px rgba(0,0,0,0.18)',
          'background:var(--color-background,#fff)',
          'color:var(--color-foreground,#111)',
          'border:1px solid var(--color-border,#e5e7eb)',
          'pointer-events:none',
        ].join(';')

        const text = document.createElement('span')
        text.style.cssText = 'overflow:hidden;text-overflow:ellipsis'
        text.textContent = props.email.subject?.trim() || props.email.from?.name || 'Email'
        el.appendChild(text)

        container.appendChild(el)
      },
    })
  },
})
</script>

<template>
  <div
    ref="emailRef"
    :class="[
      'cursor-pointer transition-opacity select-none',
      isDragging ? 'cursor-grabbing opacity-30' : '',
    ]"
    @click="emit('click', $event)"
  >
    <EmailItem
      :exclude-labels="excludeLabels"
      :is-selected="isSelected"
      v-bind="email"
    />
  </div>
</template>
