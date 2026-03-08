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
  isMultiSelected?: boolean
  selectedIds?: string[]
}>()

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void
}>()

const emailRef = ref<HTMLElement | null>(null)

const getDragData = () => {
  const isMultiDrag =
    props.selectedIds && props.selectedIds.length > 0 && props.selectedIds.includes(props.email.id)

  return {
    type: 'email' as const,
    id: props.email.id,
    accountId: props.email.account_id,
    folderId: props.email.folder_id,
    fromSwimlaneId: props.swimlaneId,
    selectedIds: isMultiDrag ? props.selectedIds : undefined,
    isMultiDrag,
  }
}

const { isDragging } = useDraggable(emailRef, getDragData, {
  onGenerateDragPreview: ({ nativeSetDragImage }) => {
    setCustomNativeDragPreview({
      nativeSetDragImage,
      getOffset: pointerOutsideOfPreview({ x: '16px', y: '8px' }),
      render({ container }) {
        const data = getDragData()
        const isMulti = data.isMultiDrag && props.selectedIds && props.selectedIds.length > 1
        const label = isMulti
          ? `${props.selectedIds!.length} emails`
          : props.email.subject?.trim() || props.email.from?.name || 'Email'

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

        if (isMulti) {
          const badge = document.createElement('span')
          badge.style.cssText =
            'background:var(--color-accent,#6366f1);color:#fff;border-radius:999px;padding:1px 7px;font-size:11px;font-weight:700;flex-shrink:0'
          badge.textContent = String(props.selectedIds!.length)
          el.appendChild(badge)
        }

        const text = document.createElement('span')
        text.style.cssText = 'overflow:hidden;text-overflow:ellipsis'
        text.textContent = label
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
      isSelected || isMultiSelected ? 'rounded bg-selection text-selection-foreground' : '',
    ]"
    @mousedown.stop
    @click.stop="emit('click', $event)"
  >
    <EmailItem
      :exclude-labels="excludeLabels"
      :is-selected="false"
      v-bind="email"
    />
  </div>
</template>
