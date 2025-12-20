<script lang="ts" setup>
import MessageView from '~/components/Ravn/MessageView.vue'
import EmailAIAnalysis from '~/components/Ravn/EmailAIAnalysis.vue'
import EmailActionButtons from '~/components/Ravn/EmailActionButtons.vue'
import type { EmailDetail } from '~/types/email'
import EmptyState from '~/components/ui/empty/EmptyState.vue'

const props = defineProps<{
  emailId: string
}>()

const { t } = useI18n()
const { fetch, archive, trash } = useEmails()
const {
  isAnalyzing,
  analysisError,
  currentAnalysis,
  analyzeEmail,
  clearAnalysisState,
  parseAnalysisFromCache
} = useCorvus()

const selectedEmail = await fetch(props.emailId)

onMounted(async () => {
  clearAnalysisState()
  const cached = parseAnalysisFromCache(selectedEmail)
  if (cached) {
    currentAnalysis.value = cached
  } else {
    try {
      await analyzeEmail(selectedEmail)
    } catch (_: unknown) {
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
  } else if (errorMsg.includes('Archive folder not found')) {
    alert(t('components.emailViewer.errors.archiveFolder'))
  } else {
    alert(`Failed to ${action.toLowerCase()}: ${errorMsg}`)
  }
}

const handleArchive = async (email: EmailDetail) => {
  try {
    await archive(email.id)
  } catch (error) {
    handleError('Archive', error)
  }
}

const handleDelete = async (email: EmailDetail) => {
  try {
    await trash(email.id)
  } catch (error) {
    handleError('Delete', error)
  }
}

const handleQuickReply = (content: string) => {
  if (!selectedEmail) return
  // Use the composable to handle quick reply with initial content
  replyTo(selectedEmail, content)
}

</script>

<template>
  <div class="flex flex-col w-full h-full">
    <div v-if="selectedEmail">
      <div class="px-3 pb-3">
        <div class="flex items-center justify-between">
          <h1 class="pl-4 text-xl font-semibold select-auto text-primary relative z-10">
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
      <MessageView
        :auto-analyze="true"
        :initial-reduced="false"
        :show-actions="false"
        v-bind="selectedEmail"
      />
    </div>
    <EmptyState
      v-else
      :description="t('components.emailViewer.emptyState.message')"
      :title="t('components.emailViewer.emptyState.title')"
      class="flex-1"
      icon="✉️"
    />
  </div>
</template>