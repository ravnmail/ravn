<script lang="ts" setup>
import AttachmentItem from '~/components/Ravn/AttachmentItem.vue'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '~/components/ui/context-menu'
import ContextMenuItemRich from '~/components/ui/context-menu/ContextMenuItemRich.vue'
import type { Attachment } from '~/types/email'

interface Props {
  attachments: Attachment[]
}

const props = defineProps<Props>()

const { t } = useI18n()
const {
  openAttachment,
  quicklookAttachments,
  saveToDownloads,
  saveToCustomLocation,
  saveMultipleToDirectory,
} = useAttachments()

const root = ref<HTMLElement | null>(null)
const itemRefs = ref<(InstanceType<typeof AttachmentItem> | HTMLElement | null)[]>([])

const selectedIds = ref<Set<string>>(new Set())
const lastSelectedIndex = ref<number | null>(null)
const focusedIndex = ref<number | null>(null)
const hasKeyboardFocusWithin = ref(false)
const isContextMenuOpen = ref(false)

const selectedAttachments = computed(() => {
  return props.attachments.filter((attachment) => selectedIds.value.has(attachment.id))
})

const hasSelection = computed(() => selectedAttachments.value.length > 0)

const getAttachmentIndex = (attachment: Attachment) => {
  return props.attachments.findIndex((item) => item.id === attachment.id)
}

const setItemRef =
  (index: number) => (el: InstanceType<typeof AttachmentItem> | HTMLElement | null) => {
    itemRefs.value[index] = el
  }

const getItemElement = (index: number) => {
  const refValue = itemRefs.value[index]

  if (!refValue) {
    return null
  }

  if (refValue instanceof HTMLElement) {
    return refValue
  }

  if ('$el' in refValue && refValue.$el instanceof HTMLElement) {
    return refValue.$el
  }

  return null
}

const syncFocusedItemTabIndex = () => {
  props.attachments.forEach((_, index) => {
    const el = getItemElement(index)
    if (!el) {
      return
    }

    el.tabIndex = focusedIndex.value === index ? 0 : -1
  })
}

const focusItem = async (index: number | null, options?: { preventScroll?: boolean }) => {
  if (index === null || index < 0 || index >= props.attachments.length) {
    return
  }

  focusedIndex.value = index
  await nextTick()
  syncFocusedItemTabIndex()
  getItemElement(index)?.focus({ preventScroll: options?.preventScroll ?? true })
}

const focusListContainer = () => {
  root.value?.focus({ preventScroll: true })
}

const clearSelection = () => {
  selectedIds.value = new Set()
  lastSelectedIndex.value = null
}

const setSingleSelection = (index: number) => {
  const attachment = props.attachments[index]
  if (!attachment) {
    return
  }

  selectedIds.value = new Set([attachment.id])
  lastSelectedIndex.value = index
  focusedIndex.value = index
}

const setRangeSelection = (index: number) => {
  if (!props.attachments[index]) {
    return
  }

  const anchorIndex = lastSelectedIndex.value ?? focusedIndex.value ?? index
  const start = Math.min(anchorIndex, index)
  const end = Math.max(anchorIndex, index)
  const nextSelectedIds = new Set<string>()

  for (let i = start; i <= end; i++) {
    nextSelectedIds.add(props.attachments[i].id)
  }

  selectedIds.value = nextSelectedIds
  focusedIndex.value = index
}

const toggleSelection = (index: number) => {
  const attachment = props.attachments[index]
  if (!attachment) {
    return
  }

  const nextSelectedIds = new Set(selectedIds.value)

  if (nextSelectedIds.has(attachment.id)) {
    nextSelectedIds.delete(attachment.id)
  } else {
    nextSelectedIds.add(attachment.id)
  }

  selectedIds.value = nextSelectedIds
  lastSelectedIndex.value = index
  focusedIndex.value = index
}

const ensureFocusedSelection = () => {
  if (focusedIndex.value === null && props.attachments.length > 0) {
    focusedIndex.value = 0
  }

  if (!hasSelection.value && focusedIndex.value !== null) {
    setSingleSelection(focusedIndex.value)
  }
}

const handleSelect = async (attachment: Attachment, event: MouseEvent) => {
  const index = getAttachmentIndex(attachment)
  if (index === -1) {
    return
  }

  if (event.shiftKey) {
    setRangeSelection(index)
  } else if (event.metaKey || event.ctrlKey) {
    toggleSelection(index)
  } else {
    setSingleSelection(index)
  }

  await focusItem(index)
}

const handleFocusItem = (attachment: Attachment) => {
  const index = getAttachmentIndex(attachment)
  if (index !== -1) {
    focusedIndex.value = index
    hasKeyboardFocusWithin.value = true
    syncFocusedItemTabIndex()
  }
}

const handleDoubleClick = async (attachment: Attachment) => {
  await openAttachment(attachment)
}

const handleQuickLook = async () => {
  ensureFocusedSelection()

  if (!hasSelection.value) {
    return
  }

  await quicklookAttachments(selectedAttachments.value)
}

const handleSaveToDownloads = async () => {
  if (!hasSelection.value) {
    return
  }

  for (const attachment of selectedAttachments.value) {
    await saveToDownloads(attachment)
  }
}

const handleSaveToCustom = async () => {
  if (!hasSelection.value) {
    return
  }

  if (selectedAttachments.value.length === 1) {
    await saveToCustomLocation(selectedAttachments.value[0])
    return
  }

  await saveMultipleToDirectory(selectedAttachments.value)
}

const handleContext = async (attachment: Attachment) => {
  const index = getAttachmentIndex(attachment)
  if (index === -1) {
    return
  }

  if (!selectedIds.value.has(attachment.id)) {
    setSingleSelection(index)
  }

  await focusItem(index)
}

const moveFocus = async (direction: 1 | -1, extendSelection = false) => {
  if (props.attachments.length === 0) {
    return
  }

  const currentIndex = focusedIndex.value ?? 0
  const nextIndex = Math.min(props.attachments.length - 1, Math.max(0, currentIndex + direction))

  if (extendSelection) {
    if (lastSelectedIndex.value === null) {
      lastSelectedIndex.value = currentIndex
    }
    setRangeSelection(nextIndex)
  } else {
    setSingleSelection(nextIndex)
  }

  await focusItem(nextIndex)
}

const focusBoundaryItem = async (index: number, extendSelection = false) => {
  if (index < 0 || index >= props.attachments.length) {
    return
  }

  if (extendSelection) {
    if (lastSelectedIndex.value === null) {
      lastSelectedIndex.value = focusedIndex.value ?? index
    }
    setRangeSelection(index)
  } else {
    setSingleSelection(index)
  }

  await focusItem(index)
}

const handleListKeydown = async (event: KeyboardEvent) => {
  if (isContextMenuOpen.value || props.attachments.length === 0) {
    return
  }

  switch (event.key) {
    case ' ':
    case 'Space':
    case 'Spacebar':
      event.preventDefault()
      event.stopPropagation()
      await handleQuickLook()
      return

    case 'ArrowRight':
    case 'ArrowDown':
      event.preventDefault()
      await moveFocus(1, event.shiftKey)
      return

    case 'ArrowLeft':
    case 'ArrowUp':
      event.preventDefault()
      await moveFocus(-1, event.shiftKey)
      return

    case 'Home':
      event.preventDefault()
      await focusBoundaryItem(0, event.shiftKey)
      return

    case 'End':
      event.preventDefault()
      await focusBoundaryItem(props.attachments.length - 1, event.shiftKey)
      return

    case 'Enter':
      event.preventDefault()
      ensureFocusedSelection()

      if (focusedIndex.value !== null) {
        await openAttachment(props.attachments[focusedIndex.value])
      }
      return

    case 'a':
    case 'A':
      if (event.metaKey || event.ctrlKey) {
        event.preventDefault()
        selectedIds.value = new Set(props.attachments.map((attachment) => attachment.id))
        lastSelectedIndex.value = 0
        if (focusedIndex.value === null) {
          focusedIndex.value = 0
        }
        await focusItem(focusedIndex.value)
      }
      return

    case 'Escape':
      event.preventDefault()
      clearSelection()
      return
  }
}

const handleListFocus = async () => {
  hasKeyboardFocusWithin.value = true

  if (props.attachments.length === 0) {
    return
  }

  if (focusedIndex.value === null) {
    focusedIndex.value =
      selectedAttachments.value.length > 0 ? getAttachmentIndex(selectedAttachments.value[0]) : 0
  }

  await nextTick()
  syncFocusedItemTabIndex()

  const activeElement = document.activeElement
  if (activeElement === root.value && focusedIndex.value !== null) {
    await focusItem(focusedIndex.value)
  }
}

const handleListFocusOut = (event: FocusEvent) => {
  const nextTarget = event.relatedTarget
  if (nextTarget instanceof Node && root.value?.contains(nextTarget)) {
    return
  }

  if (isContextMenuOpen.value) {
    return
  }

  hasKeyboardFocusWithin.value = false
}

const handlePointerDownOutside = (event: PointerEvent) => {
  if (isContextMenuOpen.value) {
    return
  }

  const target = event.target
  if (target instanceof Node && root.value?.contains(target)) {
    return
  }

  clearSelection()
  hasKeyboardFocusWithin.value = false
}

watch(
  () => props.attachments.map((attachment) => attachment.id),
  async (attachmentIds) => {
    const validIds = new Set(attachmentIds)
    const nextSelectedIds = new Set([...selectedIds.value].filter((id) => validIds.has(id)))

    if (nextSelectedIds.size !== selectedIds.value.size) {
      selectedIds.value = nextSelectedIds
    }

    itemRefs.value = itemRefs.value.slice(0, props.attachments.length)

    if (props.attachments.length === 0) {
      focusedIndex.value = null
      lastSelectedIndex.value = null
      return
    }

    if (
      focusedIndex.value === null ||
      focusedIndex.value < 0 ||
      focusedIndex.value >= props.attachments.length
    ) {
      focusedIndex.value = 0
    }

    if (
      lastSelectedIndex.value !== null &&
      (lastSelectedIndex.value < 0 || lastSelectedIndex.value >= props.attachments.length)
    ) {
      lastSelectedIndex.value = props.attachments.length - 1
    }

    await nextTick()
    syncFocusedItemTabIndex()
  },
  { immediate: true }
)

onMounted(() => {
  window.addEventListener('pointerdown', handlePointerDownOutside, { capture: true })
})

onUnmounted(() => {
  window.removeEventListener('pointerdown', handlePointerDownOutside, { capture: true })
})
</script>

<template>
  <div
    v-if="attachments.length > 0"
    ref="root"
    role="listbox"
    aria-multiselectable="true"
    :aria-label="t('components.attachments.contextMenu.saveTo')"
    tabindex="0"
    @keydown.capture="handleListKeydown"
    @focus="handleListFocus"
    @focusout="handleListFocusOut"
  >
    <ContextMenu
      @update:open="
        (open) => {
          isContextMenuOpen = open
          if (!open) {
            focusListContainer()
          }
        }
      "
    >
      <ContextMenuTrigger class="block">
        <div class="flex flex-wrap gap-1.5">
          <AttachmentItem
            v-for="(attachment, index) in attachments"
            :key="attachment.id"
            :ref="setItemRef(index)"
            :attachment="attachment"
            :selected="selectedIds.has(attachment.id)"
            :focused="focusedIndex === index"
            @focus-item="handleFocusItem"
            @contextmenu="() => handleContext(attachment)"
            @dblclick="handleDoubleClick"
            @select="(att, event) => handleSelect(att, event)"
            @key-navigate="handleListKeydown"
          />
        </div>
      </ContextMenuTrigger>

      <ContextMenuContent>
        <ContextMenuItemRich
          :label="t('components.attachments.contextMenu.quickLook')"
          icon="lucide:eye"
          shortcut="Space"
          :disabled="!hasSelection"
          @select="handleQuickLook"
        />
        <ContextMenuSeparator />
        <ContextMenuItemRich
          :label="t('components.attachments.contextMenu.saveToDownloads')"
          icon="lucide:download"
          :disabled="!hasSelection"
          @select="handleSaveToDownloads"
        />
        <ContextMenuItemRich
          :label="t('components.attachments.contextMenu.saveTo')"
          icon="lucide:folder"
          :disabled="!hasSelection"
          @select="handleSaveToCustom"
        />
      </ContextMenuContent>
    </ContextMenu>
  </div>
</template>
