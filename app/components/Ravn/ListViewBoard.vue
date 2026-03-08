<script lang="ts" setup>
import ConversationViewer from '~/components/Ravn/ConversationViewer.vue'
import MailList from '~/components/Ravn/MailList.vue'
import { useSelectedConversation } from '~/components/Ravn/view/useSelectedConversation'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '~/components/ui/resizable'
import type { View } from '~/types/view'

const props = defineProps<{
  view: View
}>()

const { selectedConversationId, selectConversation, clearSelectedConversation } =
  useSelectedConversation()

const onConversationSelect = (conversationId?: string) => {
  if (conversationId) {
    selectConversation(conversationId)
  } else {
    clearSelectedConversation()
  }
}

watch(
  () => props.view.id,
  () => {
    clearSelectedConversation()
  }
)
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden">
    <div class="flex shrink-0 items-center gap-1 border-b px-3 py-2">
      <slot />
    </div>

    <ResizablePanelGroup
      :auto-save-id="`list-view-layout-${props.view.id}`"
      class="min-h-0 flex-1 select-none"
      direction="horizontal"
    >
      <ResizablePanel
        id="list-view-mail-list-panel"
        :initial-size="300"
        :min-size="240"
        class="border-r border-border"
        size-unit="px"
      >
        <MailList
          :account-id="''"
          :conversation-id="selectedConversationId"
          :folder-id="props.view.folders?.[0] || ''"
          :view="props.view"
          class="h-full shrink-0"
          @select-conversation="onConversationSelect"
        />
      </ResizablePanel>

      <ResizableHandle />

      <ResizablePanel
        id="list-view-detail-panel"
        :min-size="40"
        class="-ml-px"
      >
        <ConversationViewer
          v-if="selectedConversationId"
          :conversation-id="selectedConversationId"
          title-class="pl-8"
        />
        <div
          v-else
          class="flex h-full items-center justify-center"
        >
          <EmptyState
            :title="'Select a conversation'"
            icon="📭"
          />
        </div>
      </ResizablePanel>
    </ResizablePanelGroup>
  </div>
</template>
