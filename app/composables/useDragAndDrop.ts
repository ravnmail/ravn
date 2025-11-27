import { draggable, dropTargetForElements } from '@atlaskit/pragmatic-drag-and-drop/element/adapter'
import type { CleanupFn } from '@atlaskit/pragmatic-drag-and-drop/types'

export interface DragData {
  type: 'email' | 'conversation' | 'folder'
  id: string
  accountId?: string
  folderId?: string
  parentId?: string | null
  // Multi-select support
  selectedIds?: string[]
  isMultiDrag?: boolean
  // Message IDs for conversation drag (contains email IDs to move)
  messageIds?: string[]

  [key: string]: unknown
}

export interface DropTargetData {
  type: 'folder' | 'swimlane'
  id: string
  accepts?: string[]

  [key: string]: unknown
}

export function useDraggable(
  element: Ref<HTMLElement | null>,
  getData: () => DragData,
  options?: {
    canDrag?: () => boolean
  }
) {
  let cleanup: CleanupFn | null = null
  const isDragging = ref(false)

  onMounted(() => {
    if (!element.value) return

    cleanup = draggable({
      element: element.value,
      getInitialData: getData,
      canDrag: options?.canDrag,
      onDragStart: () => {
        isDragging.value = true
      },
      onDrop: () => {
        isDragging.value = false
      },
    })
  })

  onUnmounted(() => {
    if (cleanup) {
      cleanup()
    }
  })

  return {
    isDragging: readonly(isDragging),
  }
}

export function useDropTarget(
  element: Ref<HTMLElement | null>,
  options: {
    getData: () => DropTargetData
    canDrop?: (data: DragData) => boolean
    onDrop: (data: DragData) => void | Promise<void>
  }
) {
  let cleanup: CleanupFn | null = null
  const isOver = ref(false)
  const canDrop = ref(false)

  onMounted(() => {
    if (!element.value) return

    cleanup = dropTargetForElements({
      element: element.value,
      getData: options.getData,
      canDrop: (args) => {
        const dragData = args.source.data as DragData
        if (options.canDrop) {
          return options.canDrop(dragData)
        }
        return true
      },
      onDragEnter: (args) => {
        const dragData = args.source.data as DragData
        isOver.value = true
        canDrop.value = !options.canDrop || options.canDrop(dragData)
      },
      onDragLeave: () => {
        isOver.value = false
        canDrop.value = false
      },
      onDrop: async (args) => {
        const dragData = args.source.data as DragData
        isOver.value = false
        canDrop.value = false
        await options.onDrop(dragData)
      },
    })
  })

  onUnmounted(() => {
    if (cleanup) {
      cleanup()
    }
  })

  return {
    isOver: readonly(isOver),
    canDrop: readonly(canDrop),
  }
}

// Multi-select support for emails/conversations
export function useMultiSelect<T extends { id: string }>() {
  const selectedIds = ref<Set<string>>(new Set())
  const lastSelectedId = ref<string | null>(null)

  const isSelected = (id: string) => {
    return computed(() => selectedIds.value.has(id))
  }

  const toggleSelect = (item: T, event?: MouseEvent) => {
    const id = item.id

    if (event?.shiftKey && lastSelectedId.value) {
      // Range select - not implemented for now, can be added later
      // Would need a list of all items to determine range
    } else if (event?.metaKey || event?.ctrlKey) {
      // Multi-select
      if (selectedIds.value.has(id)) {
        selectedIds.value.delete(id)
      } else {
        selectedIds.value.add(id)
      }
    } else {
      // Single select
      selectedIds.value.clear()
      selectedIds.value.add(id)
    }

    lastSelectedId.value = id
  }

  const clearSelection = () => {
    selectedIds.value.clear()
    lastSelectedId.value = null
  }

  const selectAll = (items: T[]) => {
    items.forEach(item => selectedIds.value.add(item.id))
  }

  const getSelectedItems = <U extends T>(items: U[]): U[] => {
    return items.filter(item => selectedIds.value.has(item.id))
  }

  const hasSelection = computed(() => selectedIds.value.size > 0)
  const selectionCount = computed(() => selectedIds.value.size)

  return {
    selectedIds: computed(() => Array.from(selectedIds.value)),
    hasSelection,
    selectionCount,
    isSelected,
    toggleSelect,
    clearSelection,
    selectAll,
    getSelectedItems,
  }
}
