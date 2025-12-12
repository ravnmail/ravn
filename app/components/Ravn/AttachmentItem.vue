<script lang="ts" setup>
import type { Attachment } from '~/types/email'

interface Props {
  attachment: Attachment
  selected?: boolean
}

const props = defineProps<Props>()

const { t } = useI18n()
const emit = defineEmits<{
  select: [attachment: Attachment, event: MouseEvent]
  contextmenu: [event: MouseEvent, attachment: Attachment]
  dblclick: [attachment: Attachment]
}>()

const { formatFileSize, getFileIcon } = useAttachments()

const handleClick = (event: MouseEvent) => {
  emit('select', props.attachment, event)
}

const handleContextMenu = (event: MouseEvent) => {
  event.preventDefault()
  emit('contextmenu', event, props.attachment)
}

const handleDoubleClick = () => {
  emit('dblclick', props.attachment)
}
</script>

<template>
  <div
    :class="[
      'flex items-center gap-3 px-3 py-2 rounded transition-colors border border-border',
      selected ? 'bg-accent border-b-accent text-accent-foreground' : 'bg-surface text-foreground',
    ]"
    @click="handleClick"
    @contextmenu="handleContextMenu"
    @dblclick="handleDoubleClick"
  >
    <Icon
      :name="getFileIcon(attachment.content_type, attachment.filename)"
      class="shrink-0"
      mode="ib"
    />
    <div class="flex-1">
      <div class="text-sm font-medium max-w-48 truncate">
        {{ attachment.filename }}
      </div>
      <div class="text-xs opacity-60 flex gap-2">
        <span>{{ formatFileSize(attachment.size) }}</span>
        <span
          v-if="!attachment.is_cached"
          class="text-xs text-yellow-500"
        >
          {{ t('components.attachment.notCached') }}
        </span>
      </div>
    </div>
  </div>
</template>
