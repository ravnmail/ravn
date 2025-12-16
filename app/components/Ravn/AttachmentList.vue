<script lang="ts" setup>
import type { Attachment } from '~/types/email'
import AttachmentItem from '~/components/Ravn/AttachmentItem.vue'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuSeparator,
  ContextMenuTrigger
} from '~/components/ui/context-menu'
import ContextMenuItemRich from '~/components/ui/context-menu/ContextMenuItemRich.vue'

interface Props {
  attachments: Attachment[]
}

const props = defineProps<Props>()

const { t } = useI18n()
const selectedIds = ref<Set<string>>(new Set())
const lastSelectedIndex = ref<number | null>(null)

const { openAttachment, quicklookAttachments, saveToDownloads, saveToCustomLocation } = useAttachments()

const selectedAttachments = computed(() => {
  return props.attachments.filter(a => selectedIds.value.has(a.id))
})

const handleSelect = (attachment: Attachment, event: MouseEvent) => {
  const index = props.attachments.findIndex(a => a.id === attachment.id)

  if (event.shiftKey && lastSelectedIndex.value !== null) {
    const start = Math.min(lastSelectedIndex.value, index)
    const end = Math.max(lastSelectedIndex.value, index)

    const newSelectedIds = new Set(selectedIds.value)
    for (let i = start; i <= end; i++) {
      newSelectedIds.add(props.attachments[i].id)
    }
    selectedIds.value = newSelectedIds
  } else if (event.metaKey || event.ctrlKey) {
    const newSelectedIds = new Set(selectedIds.value)
    if (newSelectedIds.has(attachment.id)) {
      newSelectedIds.delete(attachment.id)
    } else {
      newSelectedIds.add(attachment.id)
    }
    selectedIds.value = newSelectedIds
    lastSelectedIndex.value = index
  } else {
    selectedIds.value = new Set([attachment.id])
    lastSelectedIndex.value = index
  }
}

const handleDoubleClick = (attachment: Attachment) => {
  openAttachment(attachment)
}

const handleQuickLook = () => {
  quicklookAttachments(selectedAttachments.value)
}

const handleSaveToDownloads = async () => {
  for (const attachment of selectedAttachments.value) {
    await saveToDownloads(attachment)
  }
}

const handleSaveToCustom = async () => {
  for (const attachment of selectedAttachments.value) {
    await saveToCustomLocation(attachment)
  }
}

const handleKeydown = (event: KeyboardEvent) => {
  if (event.code === 'Space' && selectedAttachments.value.length > 0) {
    event.preventDefault()
    quicklookAttachments(selectedAttachments.value)
  }
}

const handleContext = (attachment: Attachment) => {
  if (!selectedIds.value.has(attachment.id)) {
    selectedIds.value = new Set([attachment.id])
    lastSelectedIndex.value = props.attachments.findIndex(a => a.id === attachment.id)
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

const list = useTemplateRef('list')
const contextMenu = useTemplateRef('contextMenu')
onClickOutside(list, (e) => {
  // selectedIds.value = new Set()
}, { ignore: [contextMenu] })

</script>

<template>
  <div
    v-if="attachments.length > 0"
    class="border-t border-border pt-3"
  >
    <ContextMenu>
      <ContextMenuTrigger
        ref="list"
        class="inline-flex flex-wrap gap-1"
      >
        <AttachmentItem
          v-for="attachment in attachments"
          :key="attachment.id"
          :attachment="attachment"
          :selected="selectedIds.has(attachment.id)"
          @contextmenu="() => handleContext(attachment)"
          @dblclick="handleDoubleClick"
          @select="(att, event) => handleSelect(att, event)"
        />
      </ContextMenuTrigger>
      <ContextMenuContent
        ref="contextMenu"
      >
        <ContextMenuItemRich
          :label="t('components.attachments.contextMenu.quickLook')"
          icon="lucide:eye"
          shortcut="Space"
          @select="handleQuickLook"
        />
        <ContextMenuSeparator/>
        <ContextMenuItemRich
          :label="t('components.attachments.contextMenu.saveToDownloads')"
          icon="lucide:download"
          @select="handleSaveToDownloads"
        />
        <ContextMenuItemRich
          :label="t('components.attachments.contextMenu.saveTo')"
          icon="lucide:folder"
          @select="handleSaveToCustom"
        />
      </ContextMenuContent>
    </ContextMenu>
  </div>
</template>
