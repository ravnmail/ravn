<script lang="ts" setup>
import type { Attachment } from '~/types/email'
import AttachmentItem from '~/components/Ravn/AttachmentItem.vue'

interface Props {
  attachments: Attachment[]
}

const props = defineProps<Props>()

const { t } = useI18n()
const selectedIds = ref<Set<string>>(new Set())
const lastSelectedIndex = ref<number | null>(null)
const contextMenuVisible = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })
const contextMenuAttachments = ref<Attachment[]>([])

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
  }
  else if (event.metaKey || event.ctrlKey) {
    const newSelectedIds = new Set(selectedIds.value)
    if (newSelectedIds.has(attachment.id)) {
      newSelectedIds.delete(attachment.id)
    }
    else {
      newSelectedIds.add(attachment.id)
    }
    selectedIds.value = newSelectedIds
    lastSelectedIndex.value = index
  }
  else {
    selectedIds.value = new Set([attachment.id])
    lastSelectedIndex.value = index
  }
}

const handleContextMenu = (event: MouseEvent, attachment: Attachment) => {
  if (!selectedIds.value.has(attachment.id)) {
    selectedIds.value = new Set([attachment.id])
  }

  contextMenuAttachments.value = selectedAttachments.value
  contextMenuPosition.value = { x: event.clientX, y: event.clientY }
  contextMenuVisible.value = true
}

const handleDoubleClick = (attachment: Attachment) => {
  openAttachment(attachment)
}

const handleQuickLook = () => {
  quicklookAttachments(contextMenuAttachments.value)
  contextMenuVisible.value = false
}

const handleSaveToDownloads = async () => {
  for (const attachment of contextMenuAttachments.value) {
    await saveToDownloads(attachment)
  }
  contextMenuVisible.value = false
}

const handleSaveToCustom = async () => {
  if (contextMenuAttachments.value.length === 1) {
    await saveToCustomLocation(contextMenuAttachments.value[0])
  }
  else {
    for (const attachment of contextMenuAttachments.value) {
      await saveToCustomLocation(attachment)
    }
  }
  contextMenuVisible.value = false
}

const closeContextMenu = () => {
  contextMenuVisible.value = false
}

const handleKeydown = (event: KeyboardEvent) => {
  if (event.code === 'Space' && selectedAttachments.value.length > 0) {
    event.preventDefault()
    quicklookAttachments(selectedAttachments.value)
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  window.addEventListener('click', closeContextMenu)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('click', closeContextMenu)
})
</script>

<template>
  <div
    v-if="attachments.length > 0"
    class="border-t border-border pt-3"
  >
    <div class="flex flex-wrap gap-3">
      <AttachmentItem
        v-for="attachment in attachments"
        :key="attachment.id"
        :attachment="attachment"
        :selected="selectedIds.has(attachment.id)"
        @contextmenu="handleContextMenu"
        @dblclick="handleDoubleClick"
        @select="(att, event) => handleSelect(att, event)"
      />
    </div>

    <Teleport to="body">
      <div
        v-if="contextMenuVisible"
        :style="{
          position: 'fixed',
          left: `${contextMenuPosition.x}px`,
          top: `${contextMenuPosition.y}px`,
          zIndex: 9999,
        }"
        class="bg-gray-800 border border-gray-700 rounded-lg shadow-xl py-1 min-w-[200px]"
        @click.stop
      >
        <button
          class="w-full px-4 py-2 text-left text-sm hover:bg-gray-700 flex items-center gap-2"
          @click="handleQuickLook"
        >
          <Icon
            class="w-4 h-4"
            name="lucide:eye"
          />
          <span>{{ t('components.attachments.contextMenu.quickLook') }}</span>
          <span class="ml-auto text-xs text-gray-500">{{ t('components.attachments.shortcuts.space') }}</span>
        </button>
        <div class="h-px bg-gray-700 my-1" />
        <button
          class="w-full px-4 py-2 text-left text-sm hover:bg-gray-700 flex items-center gap-2"
          @click="handleSaveToDownloads"
        >
          <Icon
            class="w-4 h-4"
            name="lucide:download"
          />
          <span>{{ t('components.attachments.contextMenu.saveToDownloads') }}</span>
        </button>
        <button
          class="w-full px-4 py-2 text-left text-sm hover:bg-gray-700 flex items-center gap-2"
          @click="handleSaveToCustom"
        >
          <Icon
            class="w-4 h-4"
            name="lucide:folder"
          />
          <span>{{ t('components.attachments.contextMenu.saveTo') }}</span>
        </button>
      </div>
    </Teleport>
  </div>
</template>
