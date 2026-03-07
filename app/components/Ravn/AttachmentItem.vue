<script lang="ts" setup>
import type { Attachment } from '~/types/email'

interface Props {
  attachment: Attachment
  selected?: boolean
  focused?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  selected: false,
  focused: false,
})

const { t } = useI18n()
const { formatFileSize, getFileIcon } = useAttachments()

const emit = defineEmits<{
  select: [attachment: Attachment, event: MouseEvent]
  dblclick: [attachment: Attachment]
  contextmenu: [attachment: Attachment, event: MouseEvent]
  focusItem: [attachment: Attachment]
  keyNavigate: [event: KeyboardEvent]
}>()

const handleClick = (event: MouseEvent) => {
  emit('select', props.attachment, event)
}

const handleDoubleClick = (event: MouseEvent) => {
  event.preventDefault()
  emit('dblclick', props.attachment)
}

const handleContextMenu = (event: MouseEvent) => {
  emit('contextmenu', props.attachment, event)
}

const handleFocus = () => {
  emit('focusItem', props.attachment)
}

const handleMouseDown = (event: MouseEvent) => {
  if (event.button === 0) {
    event.preventDefault()
  }
}

const handleKeydown = (event: KeyboardEvent) => {
  switch (event.key) {
    case ' ':
    case 'Space':
    case 'Spacebar':
      event.preventDefault()
      return

    case 'ArrowRight':
    case 'ArrowLeft':
    case 'ArrowDown':
    case 'ArrowUp':
    case 'Home':
    case 'End':
    case 'Enter':
    case 'Escape':
    case 'a':
    case 'A':
      event.stopPropagation()
      emit('keyNavigate', event)
      return
  }
}
</script>

<template>
  <div
    :class="[
      'group flex items-center gap-2 rounded-md border p-2 transition-colors outline-none select-none',
      selected
        ? 'border-border/80 bg-muted/40 text-foreground'
        : 'border-border bg-surface text-foreground hover:bg-muted/30',
      focused ? 'ring-1 ring-primary/35 ring-offset-1 ring-offset-background' : '',
    ]"
    role="option"
    :aria-selected="selected ? 'true' : 'false'"
    :title="attachment.filename"
    :tabindex="focused ? 0 : -1"
    @focus="handleFocus"
    @mousedown="handleMouseDown"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @contextmenu="handleContextMenu"
    @keydown="handleKeydown"
  >
    <Icon
      :name="getFileIcon(attachment.content_type, attachment.filename)"
      :size="24"
      class="shrink-0 opacity-90"
      mode="ib"
    />

    <div class="min-w-0 flex-1">
      <div class="max-w-48 truncate text-sm font-medium">
        {{ attachment.filename }}
      </div>

      <div class="flex flex-wrap gap-2 text-xs opacity-60">
        <span>{{ formatFileSize(attachment.size) }}</span>
        <span
          v-if="!attachment.is_cached"
          class="text-warning"
        >
          {{ t('components.attachment.notCached') }}
        </span>
      </div>
    </div>
  </div>
</template>
