<script lang="ts" setup>
import type { EmailListItem } from '~/types/email'
import EmailItem from '~/components/Ravn/EmailItem.vue'
import { useDraggable } from '~/composables/useDragAndDrop'

const props = defineProps<{
  email: EmailListItem
  swimlaneId: string
  excludeLabels?: string[]
}>()

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void
}>()

const emailRef = ref<HTMLElement | null>(null)

const { isDragging } = useDraggable(emailRef, () => ({
  type: 'email',
  id: props.email.id,
  accountId: props.email.account_id,
  folderId: props.email.folder_id,
  fromSwimlaneId: props.swimlaneId,
}))
</script>

<template>
  <div
    ref="emailRef"
    :class="['transition-opacity', isDragging ? 'opacity-50' : 'opacity-100']"
    @click="emit('click', $event)"
  >
    <EmailItem
      :exclude-labels="excludeLabels"
      v-bind="email"
    />
  </div>
</template>
