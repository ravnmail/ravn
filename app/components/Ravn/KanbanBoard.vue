<script lang="ts" setup>
import ConversationViewer from '~/components/Ravn/ConversationViewer.vue'
import KanbanSwimlane from '~/components/Ravn/KanbanSwimlane.vue'
import { useSelectedConversation } from '~/components/Ravn/view/useSelectedConversation'
import { Button } from '~/components/ui/button'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import { ScrollArea } from '~/components/ui/scroll-area'
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import type { DragData } from '~/composables/useDragAndDrop'
import type { EmailListItem } from '~/types/email'
import type { KanbanSwimlane as KanbanSwimlaneType, KanbanViewConfig, View } from '~/types/view'

const props = defineProps<{
  view: View
}>()

const { t } = useI18n()
const { updateView } = useViews()
const {
  addLabelToEmail,
  removeLabelFromEmail,
  fetchForFolder,
  fetchForLabels,
  isLoading: isLoadingEmails,
  move,
} = useEmails()

const swimlaneEmails = ref<Record<string, EmailListItem[]>>({})
const swimlanes = computed(() => (props.view.config as KanbanViewConfig).swimlanes || [])
const usedLabelIds = computed(() => swimlanes.value.flatMap((s) => s.label_ids || []))

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
        emails.push(
          ...folderEmails.filter((e) => {
            const emailLabelIds = e.labels.map((l) => l.id)
            return !emailLabelIds.some(
              (labelId) =>
                usedLabelIds.value.includes(labelId) &&
                !(swimlane.label_ids || []).includes(labelId)
            )
          })
        )
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

  const email = swimlaneEmails.value[dragData.fromSwimlaneId]?.find(
    (e) => e.id === dragData.id
  ) as EmailListItem
  if (!email) return

  const fromSwimlane = swimlanes.value.find((s) => s.id === dragData.fromSwimlaneId)
  const toSwimlane = swimlanes.value.find((s) => s.id === targetSwimlaneId)

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

const { selectedConversationId, selectConversation, clearSelectedConversation } =
  useSelectedConversation()

const handleSwimlaneUpdate = async (updatedSwimlane: KanbanSwimlaneType) => {
  console.log('Updating swimlane:', updatedSwimlane)

  const config = JSON.parse(JSON.stringify(props.view.config)) as KanbanViewConfig
  const index = config.swimlanes.findIndex((s) => s.id === updatedSwimlane.id)
  if (index !== -1) {
    config.swimlanes[index] = updatedSwimlane
    try {
      await updateView({
        ...props.view,
        config,
      })
    } catch (error) {
      console.error('Failed to update swimlane:', error)
    }
  }
}

const select = (email: EmailListItem) => {
  selectConversation(email.conversation_id)
}

const onSheetClose = () => {
  clearSelectedConversation()
}
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <div class="flex items-center gap-1 p-3">
      <slot />
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
    <ScrollArea class="h-full flex-1">
      <div class="flex h-full w-full flex-1 gap-4 p-4">
        <KanbanSwimlane
          v-for="swimlane in swimlanes"
          :key="swimlane.id"
          :emails="swimlaneEmails[swimlane.id] || []"
          :selected-conversation-id="selectedConversationId"
          :swimlane="swimlane"
          @drop="handleEmailDrop"
          @update="handleSwimlaneUpdate"
          @email-click="select"
          @refresh="loadEmails"
        />
        <EmptyState
          v-if="swimlanes.length === 0"
          :description="t('components.kanban.emptyState.noSwimlanes.message')"
          :title="t('components.kanban.emptyState.noSwimlanes.title')"
          class="h-full"
          icon-name="lucide:layout-dashboard"
        />
      </div>
    </ScrollArea>
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
