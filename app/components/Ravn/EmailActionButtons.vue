<script lang="ts" setup>
import { Button } from '~/components/ui/button'
import type { EmailDetail, EmailListItem } from '~/types/email'

const props = withDefaults(defineProps<{
  email: EmailDetail | EmailListItem
  size?: 'default' | 'sm' | 'lg' | 'icon'
  showDivider?: boolean
}>(), {
  size: 'icon',
  showDivider: true
})

const emit = defineEmits<{
  (e: 'reply' | 'reply-all' | 'forward' | 'archive' | 'delete', email: EmailDetail | EmailListItem): void
}>()

const { t } = useI18n()
const { archive, trash } = useEmails()

const isArchiving = ref(false)
const isDeleting = ref(false)

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

const handleReply = () => {
  emit('reply', props.email)
}

const handleReplyAll = () => {
  emit('reply-all', props.email)
}

const handleForward = () => {
  emit('forward', props.email)
}

const handleArchive = async () => {
  if (isArchiving.value) return

  isArchiving.value = true
  try {
    await archive(props.email.id)
    emit('archive', props.email)
  }
  catch (error) {
    handleError('Archive', error)
  }
  finally {
    isArchiving.value = false
  }
}

const handleDelete = async () => {
  if (isDeleting.value) return

  isDeleting.value = true
  try {
    await trash(props.email.id)
    emit('delete', props.email)
  }
  catch (error) {
    handleError('Delete', error)
  }
  finally {
    isDeleting.value = false
  }
}
</script>

<template>
  <div class="flex items-center space-x-2">
    <Button
      :size="size"
      :title="t('components.emailActions.reply.tooltip')"
      variant="ghost"
      @click="handleReply"
    >
      <Icon name="lucide:reply" />
    </Button>
    <Button
      :size="size"
      :title="t('components.emailActions.replyAll.tooltip')"
      variant="ghost"
      @click="handleReplyAll"
    >
      <Icon name="lucide:reply-all" />
    </Button>
    <Button
      :size="size"
      :title="t('components.emailActions.forward.tooltip')"
      variant="ghost"
      @click="handleForward"
    >
      <Icon name="lucide:forward" />
    </Button>
    <div
      v-if="showDivider"
      class="w-px h-6 bg-border"
    />
    <Button
      :disabled="isArchiving"
      :size="size"
      :title="t('components.emailActions.archive.tooltip')"
      variant="ghost"
      @click="handleArchive"
    >
      <Icon
        :class="{ 'animate-pulse': isArchiving }"
        name="lucide:archive"
      />
    </Button>
    <Button
      :disabled="isDeleting"
      :size="size"
      :title="t('components.emailActions.delete.tooltip')"
      variant="ghost"
      @click="handleDelete"
    >
      <Icon
        :class="{ 'animate-pulse': isDeleting }"
        name="lucide:trash-2"
      />
    </Button>
  </div>
</template>
