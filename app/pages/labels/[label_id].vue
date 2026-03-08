<script lang="ts" setup>
import MailList from '~/components/Ravn/MailList.vue'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '~/components/ui/resizable'

const route = useRoute()
const labelId = computed(() => route.params.label_id as string)
const conversationId = computed(() => route.params.conversation as string)

const { useGetLabel } = useLabels()
const { data: labelData } = useGetLabel(labelId)

const labelView = computed(() => {
  const label = labelData.value

  return {
    id: `label-${labelId.value}`,
    name: label?.name || '',
    icon: label?.icon || 'tag',
    color: label?.color,
    view_type: 'list' as const,
    config: {
      type: 'list' as const,
      folder_ids: [],
      label_ids: [labelId.value],
      match_all_labels: false,
    },
    folders: [],
    sort_order: 0,
    is_default: false,
    created_at: '',
    updated_at: '',
  }
})
</script>

<template>
  <ResizablePanelGroup
    auto-save-id="label-view-layout"
    class="flex select-none"
    direction="horizontal"
  >
    <ResizablePanel
      id="label-mail-list-panel"
      :initial-size="300"
      :min-size="240"
      class="border-r border-border"
      size-unit="px"
    >
      <MailList
        :account-id="''"
        :conversation-id="conversationId"
        :folder-id="''"
        :view="labelView"
        class="h-full shrink-0"
      />
    </ResizablePanel>
    <ResizableHandle />
    <ResizablePanel
      id="label-mail-viewer-panel"
      :min-size="50"
      class="-ml-px"
    >
      <NuxtPage />
    </ResizablePanel>
  </ResizablePanelGroup>
</template>
