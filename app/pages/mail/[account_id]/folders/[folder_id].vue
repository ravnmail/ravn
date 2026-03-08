<script lang="ts" setup>
import MailList from '~/components/Ravn/MailList.vue'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '~/components/ui/resizable'

const route = useRoute()
const router = useRouter()
const folderId = computed(() => route.params.folder_id as string)
const accountId = computed(() => route.params.account_id as string)
const conversationId = computed(() => route.params.conversation as string)

const onConversationSelect = (nextConversationId?: string) => {
  if (nextConversationId) {
    router.push(
      `/mail/${accountId.value}/folders/${folderId.value}/conversations/${nextConversationId}`
    )
    return
  }

  router.push(`/mail/${accountId.value}/folders/${folderId.value}`)
}
</script>

<template>
  <ResizablePanelGroup
    auto-save-id="folder-view-layout"
    class="flex select-none"
    direction="horizontal"
  >
    <ResizablePanel
      id="mail-list-panel"
      :initial-size="300"
      :min-size="240"
      class="border-r border-border"
      size-unit="px"
    >
      <MailList
        :account-id="accountId"
        :conversation-id="conversationId"
        :folder-id="folderId"
        class="h-full shrink-0"
        @select-conversation="onConversationSelect"
      />
    </ResizablePanel>
    <ResizableHandle />
    <ResizablePanel
      id="mail-viewer-panel"
      :min-size="50"
      class="-ml-px"
    >
      <NuxtPage />
    </ResizablePanel>
  </ResizablePanelGroup>
</template>
