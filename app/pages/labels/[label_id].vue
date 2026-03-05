<script lang="ts" setup>
import LabelMailList from '~/components/Ravn/LabelMailList.vue'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '~/components/ui/resizable'

const route = useRoute()
const labelId = computed(() => route.params.label_id as string)
const conversationId = computed(() => route.params.conversation as string)
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
      <LabelMailList
        :label-id="labelId"
        :conversation-id="conversationId"
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
