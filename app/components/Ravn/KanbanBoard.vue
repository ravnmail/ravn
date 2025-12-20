<script lang="ts" setup>
import type { KanbanViewConfig, View } from '~/types/view'
import type { EmailListItem } from '~/types/email'
import { Button } from '~/components/ui/button'
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import ConversationViewer from '~/components/Ravn/ConversationViewer.vue'
import { HorizontalScrollArea, ScrollArea } from '~/components/ui/scroll-area'
import KanbanSwimlane from '~/components/Ravn/KanbanSwimlane.vue'
import type { DragData } from '~/composables/useDragAndDrop'
import EmptyState from '~/components/ui/empty/EmptyState.vue'

const props = defineProps<{
  view: View
}>()

const { t } = useI18n()
const {
  addLabelToEmail,
  removeLabelFromEmail,
  fetchForFolder,
  fetchForLabels,
  isLoading: isLoadingEmails,
  move
} = useEmails()

const swimlaneEmails = ref<Record<string, EmailListItem[]>>({})
const swimlanes = computed(() => (props.view.config as KanbanViewConfig).swimlanes || [])
const usedLabelIds = computed(() => swimlanes.value.flatMap(s => s.label_ids || []))

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

const loadEmails = async () => {
  for (const swimlane of swimlanes.value) {
    let emails: EmailListItem[] = []

    if (swimlane.folder_ids && swimlane.folder_ids.length > 0) {
      for (const folderId of swimlane.folder_ids) {
        const folderEmails = await fetchForFolder(folderId, 100, 0)
        emails.push(...(folderEmails.filter(e => {
          const emailLabelIds = e.labels.map(l => l.id)
          return !emailLabelIds.some(labelId => usedLabelIds.value.includes(labelId) && !(swimlane.label_ids || []).includes(labelId))
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

  const email = swimlaneEmails.value[dragData.fromSwimlaneId]?.find(e => e.id === dragData.id) as EmailListItem
  if (!email) return

  const fromSwimlane = swimlanes.value.find(s => s.id === dragData.fromSwimlaneId)
  const toSwimlane = swimlanes.value.find(s => s.id === targetSwimlaneId)

  if (!toSwimlane || dragData.fromSwimlaneId === targetSwimlaneId) {
    return
  }

  try {
    const isToSwimlaneFolder = toSwimlane.folder_ids && toSwimlane.folder_ids.length > 0
    const isToSwimlaneLabel = toSwimlane.label_ids && toSwimlane.label_ids.length > 0
    const emailLabelIds = email.labels.map(({ id }) => id)

    if (isToSwimlaneFolder) {
      for (const labelId of emailLabelIds) {
        if (usedLabelIds.value.includes(labelId)) {
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
        if (usedLabelIds.value.includes(labelId)) {
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
const selectedConversationId = computed({
  get() {
    return route.query.conversation as string | undefined
  },
  set(value: string | undefined) {
    const query = { ...route.query }
    if (value) {
      query.conversation = value
    } else {
      delete query.conversation
    }
    router.replace({ query })
  }
})

const select = (email: EmailListItem) => {
  selectedConversationId.value = email.conversation_id
}

const onSheetClose = () => {
  selectedConversationId.value = undefined
}
</script>

<template>
  <div class="h-full w-full flex flex-col">
    <div class="flex items-center p-3 gap-1">
      <slot/>
      <Button
        size="sm"
        variant="ghost"
        @click="handleManualRefresh"
      >
        <Icon
          :class="{ 'animate-spin': isLoadingEmails }"
          class="h-4 w-4"
          name="lucide:refresh-cw"
        />
      </Button>
    </div>
    <HorizontalScrollArea class="h-full flex-1">
      <div class="flex flex-1 w-full h-full gap-4 p-4">
        <KanbanSwimlane
          v-for="swimlane in swimlanes"
          :key="swimlane.id"
          :emails="swimlaneEmails[swimlane.id] || []"
          :swimlane="swimlane"
          @drop="handleEmailDrop"
          @email-click="select"
        />
        <EmptyState
          v-if="swimlanes.length === 0"
          :description="t('components.kanban.emptyState.noSwimlanes.message')"
          :title="t('components.kanban.emptyState.noSwimlanes.title')"
          class="h-full"
          icon-name="lucide:layout-dashboard"
        />
      </div>
    </HorizontalScrollArea>
    <UnobstrusiveSheetContent
      v-if="selectedConversationId"
      @close="onSheetClose()"
    >
      <ConversationViewer
        :conversation-id="selectedConversationId"
        title-class="pl-8"
      />
    </UnobstrusiveSheetContent>
  </div>
</template>