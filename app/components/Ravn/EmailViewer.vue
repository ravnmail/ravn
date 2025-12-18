<script lang="ts" setup>
import MessageView from '~/components/Ravn/MessageView.vue'
import EmailAIAnalysis from '~/components/Ravn/EmailAIAnalysis.vue'
import EmailActionButtons from '~/components/Ravn/EmailActionButtons.vue'
import type { EmailDetail } from '~/types/email'

const props = defineProps<{
  emailId: string
}>()

const { t } = useI18n()
const { fetch, archive, trash } = useEmails()
const { isAnalyzing, analysisError, currentAnalysis, analyzeEmail, clearAnalysisState, parseAnalysisFromCache } = useCorvus()

const selectedEmail = await fetch(props.emailId)

onMounted(async () => {
  clearAnalysisState()
  const cached = parseAnalysisFromCache(selectedEmail)
  if (cached) {
    currentAnalysis.value = cached
  }
  else {
    try {
      await analyzeEmail(selectedEmail)
    }
    catch (_: unknown) {
      // Ignore analysis errors
    }
  }
})

const handleReply = (email: EmailDetail) => {
  replyTo(email)
}

const handleReplyAll = (email: EmailDetail) => {
  // For now, reuse replyTo - can be enhanced later for explicit reply-all
  replyTo(email)
}

const handleForward = (email: EmailDetail) => {
  forwardEmail(email)
}

const handleError = (action: string, error: any) => {
  const errorMsg = error instanceof Error ? error.message : String(error)

  if (errorMsg.includes('IMAP config not set') || errorMsg.includes('credentials')) {
    alert(t('components.emailViewer.errors.credentials'))
  }
  else if (errorMsg.includes('Archive folder not found')) {
    alert(t('components.emailViewer.errors.archiveFolder'))
  }
  else {
    alert(`Failed to ${action.toLowerCase()}: ${errorMsg}`)
  }
}

const handleArchive = async (email: EmailDetail) => {
  try {
    await archive(email.id)
  }
  catch (error) {
    handleError('Archive', error)
  }
}

const handleDelete = async (email: EmailDetail) => {
  try {
    await trash(email.id)
  }
  catch (error) {
    handleError('Delete', error)
  }
}

const handleQuickReply = (content: string) => {
  if (!selectedEmail) return
  // Use the composable to handle quick reply with initial content
  replyTo(selectedEmail, content)
}


// watch(selectedEmail, (newEmail) => {
//   if (newEmail && !newEmail.is_read) {
//     setTimeout(() => {
//       if (selectedEmail?.id === newEmail.id && !newEmail.is_read) {
//         markEmailAsRead(newEmail, newEmail.account_id, { showNotification: false })
//           .catch(_ => {
//             // Ignore errors here
//           })
//       }
//     }, 1000)
//   }
// })
//
// watch(selectedEmail, async (newEmail, oldEmail) => {
//   if (!newEmail || newEmail.id === oldEmail?.id) {
//     return
//   }
//
//   clearAnalysisState()
//   const cached = parseAnalysisFromCache(newEmail)
//   if (cached) {
//     currentAnalysis.value = cached
//   }
//   else {
//     try {
//       await analyzeEmail(newEmail)
//     }
//     catch (_: unknown) {
//       // Ignore analysis errors
//     }
//   }
//
//   if (newEmail.has_attachments) {
//     await loadAttachments(newEmail.id)
//   }
// }, { immediate: true })

</script>

<template>
  <div class="flex flex-col">
    <div
      v-if="selectedEmail"
      class="flex-1"
    >
      <div class="p-3 border-b border-border">
        <div class="flex items-center justify-between">
          <h1 class="text-2xl font-semibold select-auto text-primary">
            {{ selectedEmail.subject || t('components.emailViewer.noSubject') }}
          </h1>
          <EmailActionButtons
            :email="selectedEmail"
            @archive="handleArchive"
            @delete="handleDelete"
            @forward="handleForward"
            @reply="handleReply"
            @reply-all="handleReplyAll"
          />
        </div>
      </div>
      <div class="py-6 space-y-3">
        <MessageView
          :auto-analyze="true"
          :initial-reduced="false"
          :show-actions="false"
          v-bind="selectedEmail"
        />
      </div>
    </div>
    <div
      v-else
      class="flex-1 flex items-center justify-center"
    >
      <div class="text-center">
        <Icon
          class="w-24 h-24 text-gray-300 mx-auto mb-4"
          name="lucide:mail"
        />
        <h3 class="text-xl font-medium text-gray-600 mb-2">
          {{ t('components.emailViewer.emptyState.title') }}
        </h3>
        <p class="text-gray-500">
          {{ t('components.emailViewer.emptyState.message') }}
        </p>
      </div>
    </div>
  </div>
</template>