<script lang="ts" setup>
import ConversationViewer from '~/components/Ravn/ConversationViewer.vue'

const route = useRoute()
const conversationId = route.params.conversation as string
const selectedEmailId = computed(() => route.query.email as string | undefined)
</script>

<template>
  <ConversationViewer
    :conversation-id="conversationId"
    :selected-email-id="selectedEmailId"
    title-class="pt-1"
  />
</template>
