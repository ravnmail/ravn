<script lang="ts" setup>
import ConversationViewer from '~/components/Ravn/ConversationViewer.vue'

const conversationId = useRoute().params.conversation as string
</script>

<template>
  <ConversationViewer
    :conversation-id="conversationId"
    title-class="pt-1"
  />
</template>
