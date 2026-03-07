<script lang="ts" setup>
import Composer from '~/components/Composer.vue'
import EmailActionButtons from '~/components/Ravn/EmailActionButtons.vue'
import MessageView from '~/components/Ravn/MessageView.vue'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import type { EmailDetail } from '~/types/email'

const props = defineProps<{
  emailId: string
}>()

const { t } = useI18n()
const { fetch, archive, trash } = useEmails()

const selectedEmail = await fetch(props.emailId)

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
  // Quick reply should open a reply-all composer with the generated content
  replyAll(selectedEmail, content)
}
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <div
      v-if="selectedEmail && selectedEmail.is_draft"
      class="p-3"
    >
      <Composer :draft="selectedEmail" />
    </div>
    <div v-else-if="selectedEmail">
      <div class="px-3 pb-3">
        <div class="flex items-center justify-between">
          <h1 class="relative z-10 pl-4 text-xl font-semibold text-primary select-auto">
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
