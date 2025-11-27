<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import type { View } from '~/types/view'
import type { EmailListItem } from '~/types/email'
import { Button } from '~/components/ui/button'
import { Sheet, SheetContent } from '~/components/ui/sheet'
import ConversationViewer from '~/components/Ravn/ConversationViewer.vue'
import { HorizontalScrollArea } from '~/components/ui/scroll-area'
import KanbanSwimlane from '~/components/Ravn/KanbanSwimlane.vue'
import type { DragData } from '~/composables/useDragAndDrop'

const props = defineProps<{
  view: View
}>()

const { t } = useI18n()
const {  addLabelToEmail, removeLabelFromEmail, fetchForFolder, fetchForLabels, isLoading: isLoadingEmails, move } = useEmails()

const swimlaneEmails = ref<Record<string, EmailListItem[]>>({})

const swimlanes = computed(() => {
  if (props.view.config.type === 'kanban') {
    return props.view.config.swimlanes
  }
  return []
})

const REFRESH_INTERVAL = 30000
const swimlaneRefreshIntervals = new Map<string, NodeJS.Timeout>()

const startAutoRefreshForSwimlane = (swimlaneId: string) => {
  stopAutoRefreshForSwimlane(swimlaneId)
  const interval = setInterval(async () => {
    await loadEmailsForSwimlane(swimlaneId)
  }, REFRESH_INTERVAL)
  swimlaneRefreshIntervals.set(swimlaneId, interval)
}

const stopAutoRefreshForSwimlane = (swimlaneId: string) => {
  const interval = swimlaneRefreshIntervals.get(swimlaneId)
  if (interval) {
    clearInterval(interval)
    swimlaneRefreshIntervals.delete(swimlaneId)
  }
}

const startAutoRefresh = () => {
  stopAutoRefresh()
  for (const swimlane of swimlanes.value) {
    startAutoRefreshForSwimlane(swimlane.id)
  }
}

const stopAutoRefresh = () => {
  swimlaneRefreshIntervals.forEach((interval, swimlaneId) => {
    stopAutoRefreshForSwimlane(swimlaneId)
  })
}

onMounted(async () => {
  await loadEmails()
  startAutoRefresh()
})

onUnmounted(() => {
  stopAutoRefresh()
})

watch(() => props.view, async () => {
  await loadEmails()
  startAutoRefresh()
})

const loadEmails = async () => {
  const usedLabelIds = swimlanes.value.flatMap(s => s.label_ids || [])
  for (const swimlane of swimlanes.value) {
    console.log(swimlane)
    let emails: EmailListItem[] = []

    if (swimlane.folder_ids && swimlane.folder_ids.length > 0) {
      for (const folderId of swimlane.folder_ids) {
        console.log(`Loading emails for folder ${folderId} in swimlane ${swimlane.id}`)
        const folderEmails = await fetchForFolder(folderId, 100, 0)
        emails.push(...(folderEmails.filter(e => {
          const emailLabelIds = e.labels.map(l => l.id)
          return !emailLabelIds.some(labelId => usedLabelIds.includes(labelId) && !(swimlane.label_ids || []).includes(labelId))
        })))
      }
    }
    if (swimlane.label_ids && swimlane.label_ids.length > 0) {
      emails = await fetchForLabels(swimlane.label_ids)
    }

    swimlaneEmails.value[swimlane.id] = emails
  }
}

const loadEmailsForSwimlane = async (swimlaneId: string) => {
  await loadEmails()
}

const handleEmailDrop = async (dragData: DragData, targetSwimlaneId: string) => {
  if (dragData.type !== 'email') return

  const email = swimlaneEmails.value[dragData.fromSwimlaneId]?.find(e => e.id === dragData.id)
  if (!email) return

  const fromSwimlane = swimlanes.value.find(s => s.id === dragData.fromSwimlaneId)
  const toSwimlane = swimlanes.value.find(s => s.id === targetSwimlaneId)

  if (!toSwimlane || dragData.fromSwimlaneId === targetSwimlaneId) {
    return
  }

  try {
    const isToSwimlaneFolder = toSwimlane.folder_ids && toSwimlane.folder_ids.length > 0
    const isToSwimlaneLabel = toSwimlane.label_ids && toSwimlane.label_ids.length > 0

    const allViewLabels = new Set<string>()
    swimlanes.value.forEach(swimlane => {
      if (swimlane.label_ids && swimlane.label_ids.length > 0) {
        swimlane.label_ids.forEach(labelId => allViewLabels.add(labelId))
      }
    })

    const emailLabelIds = email.labels.map(l => l.id)

    if (isToSwimlaneFolder) {
      for (const labelId of emailLabelIds) {
        if (allViewLabels.has(labelId)) {
          await removeLabelFromEmail(email.id, labelId)
        }
      }

      const targetFolderId = toSwimlane.folder_ids[0]
      if (email.folder_id !== targetFolderId) {
        await move(email.id, targetFolderId)
      }
    } else if (isToSwimlaneLabel) {
      if (fromSwimlane && fromSwimlane.label_ids && fromSwimlane.label_ids.length > 0) {
        for (const labelId of fromSwimlane.label_ids) {
          await removeLabelFromEmail(email.id, labelId)
        }
      }

      for (const labelId of toSwimlane.label_ids) {
        if (!emailLabelIds.includes(labelId)) {
          await addLabelToEmail({ email_id: email.id, label_id: labelId })
        }
      }
    } else {
      for (const labelId of emailLabelIds) {
        if (allViewLabels.has(labelId)) {
          await removeLabelFromEmail(email.id, labelId)
        }
      }
    }

    await loadEmails()
  } catch (error) {
    console.error('Failed to move email:', error)
  }
}

const handleManualRefresh = async () => {
  await loadEmails()
  startAutoRefresh()
}

const router = useRouter()
const route = useRoute()

const selectedEmailId = computed({
  get() {
    return route.query.email as string | undefined
  },
  set(value: string | undefined) {
    const query = { ...route.query }
    if (value) {
      query.email = value
    } else {
      delete query.email
    }
    router.replace({ query })
  }
})

const selectedEmail = computed(() => {
  if (!selectedEmailId.value) return null
  for (const swimlaneId in swimlaneEmails.value) {
    const email = swimlaneEmails.value[swimlaneId]?.find(e => e.id === selectedEmailId.value)
    if (email) return email
  }
  return null
})

const selectedConversationId = computed(() => {
  return selectedEmail.value?.conversation_id || selectedEmail.value?.id
})

const select = (email: EmailListItem) => {
  router.replace(`/views/${route.params.view}?email=${email.id}`)
}

const onSheetChange = (e) => {
  if (!e) {
    selectedEmailId.value = undefined
  }
}

const handleInteractOutside = (event: Event) => {
  const target = event.target as HTMLElement

  if (target.closest('.tippy-box, .tippy-content, [data-tippy-root]')) {
    event.preventDefault()
    return
  }

  if (target.closest('[role="toolbar"], [role="menu"]')) {
    event.preventDefault()
    return
  }
}

</script>

<template>
  <div class="h-full w-full flex flex-col">
    <div class="flex items-center justify-between p-4">
      <div/>
      <div class="flex items-center gap-2">
        <Button
          size="sm"
          variant="outline"
          @click="handleManualRefresh"
        >
          <Icon
            :class="{ 'animate-spin': isLoadingEmails }"
            class="h-4 w-4"
            name="lucide:refresh-cw"
          />
        </Button>
      </div>
    </div>
    <HorizontalScrollArea>
      <div class="flex flex-1 w-full h-full gap-4 p-4">
        <KanbanSwimlane
          v-for="swimlane in swimlanes"
          :key="swimlane.id"
          :emails="swimlaneEmails[swimlane.id] || []"
          :swimlane="swimlane"
          @drop="handleEmailDrop"
          @email-click="select"
        />
        <div
          v-if="swimlanes.length === 0"
          class="flex-1 flex items-center justify-center"
        >
          <div class="text-center text-gray-500">
            <Icon
              class="h-16 w-16 mx-auto mb-4 opacity-30"
              name="lucide:layout-dashboard"
            />
            <p class="text-lg font-medium mb-2">{{ t('components.kanban.emptyState.noSwimlanes.title') }}</p>
            <p class="text-sm">{{ t('components.kanban.emptyState.noSwimlanes.message') }}</p>
          </div>
        </div>
      </div>
    </HorizontalScrollArea>
    <Sheet
      :open="!!selectedEmailId"
      side="right"
      @update:open="onSheetChange"
    >
      <SheetContent
        class="w-full sm:max-w-4xl"
        @interact-outside="handleInteractOutside"
      >
        <ConversationViewer
          v-if="selectedConversationId"
          :conversation-id="selectedConversationId"
        />
      </SheetContent>
    </Sheet>
  </div>
</template>