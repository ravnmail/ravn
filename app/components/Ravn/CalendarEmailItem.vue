<script lang="ts" setup>
import MailContextMenu from '~/components/Ravn/MailContextMenu.vue'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import { useDraggable } from '~/composables/useDragAndDrop'
import { useRegionalFormat } from '~/composables/useFormatting'
import type { EmailListItem } from '~/types/email'

const props = defineProps<{
  email: EmailListItem
  isSelected?: boolean
}>()

const emit = defineEmits<{
  (e: 'click'): void
  (e: 'action', actionId: string, arg?: unknown): void
}>()

const { t } = useI18n()
const { archive, trash, move, updateRead, addLabelToEmail, setRemindAt } = useEmails()
const { formatTime } = useRegionalFormat()

const itemRef = ref<HTMLElement | null>(null)

const { isDragging } = useDraggable(itemRef, () => ({
  type: 'email',
  id: props.email.id,
  folderId: props.email.folder_id,
}))

const executeAction = async (actionId: string, arg?: unknown) => {
  const email = props.email
  switch (actionId) {
    case 'archiveEmail':
      await archive(email.id)
      emit('action', 'archiveEmail')
      break
    case 'deleteEmail':
      await trash(email.id)
      emit('action', 'deleteEmail')
      break
    case 'moveEmail':
      await move(email.id, arg as string)
      emit('action', 'moveEmail', arg)
      break
    case 'markRead':
      await updateRead(email.id, true)
      emit('action', 'markRead')
      break
    case 'markUnread':
      await updateRead(email.id, false)
      emit('action', 'markUnread')
      break
    case 'assignLabel':
      await addLabelToEmail({ email_id: email.id, label_id: arg as string })
      emit('action', 'assignLabel', arg)
      break
    case 'setRemindAt':
      await setRemindAt(email.id, (arg as string | null) ?? null)
      emit('action', 'setRemindAt', arg)
      break
  }
}

interface ReminderPreset {
  labelKey: string
  icon: string
  getValue: () => string | null
}

const reminderPresets = computed<ReminderPreset[]>(() => {
  const now = new Date()

  const laterToday = new Date(now)
  laterToday.setHours(now.getHours() + 3, 0, 0, 0)

  const tomorrow = new Date(now)
  tomorrow.setDate(now.getDate() + 1)
  tomorrow.setHours(9, 0, 0, 0)

  const nextWeek = new Date(now)
  nextWeek.setDate(now.getDate() + 7)
  nextWeek.setHours(9, 0, 0, 0)

  const nextMonth = new Date(now)
  nextMonth.setMonth(now.getMonth() + 1)
  nextMonth.setDate(1)
  nextMonth.setHours(9, 0, 0, 0)

  return [
    {
      labelKey: 'components.remindAt.laterToday',
      icon: 'lucide:clock-3',
      getValue: () => laterToday.toISOString(),
    },
    {
      labelKey: 'components.remindAt.tomorrow',
      icon: 'lucide:sunrise',
      getValue: () => tomorrow.toISOString(),
    },
    {
      labelKey: 'components.remindAt.nextWeek',
      icon: 'lucide:calendar-days',
      getValue: () => nextWeek.toISOString(),
    },
    {
      labelKey: 'components.remindAt.nextMonth',
      icon: 'lucide:calendar-range',
      getValue: () => nextMonth.toISOString(),
    },
    {
      labelKey: 'components.remindAt.clear',
      icon: 'lucide:x-circle',
      getValue: () => null,
    },
  ]
})

const dropdownOpen = ref(false)

const handleFolderSelect = (v: string | string[]) => {
  const id = Array.isArray(v) ? v[0] : v
  if (!id) return
  executeAction('moveEmail', id)
  nextTick(() => {
    dropdownOpen.value = false
  })
}

const timeLabel = computed(() => {
  const raw = props.email.sent_at || props.email.received_at
  return formatTime(raw)
})

const hasReminder = computed(() => !!props.email.remind_at)
</script>

<template>
  <MailContextMenu
    :selected-email-ids="[email.id]"
    :on-execute-action="executeAction"
  >
    <div
      ref="itemRef"
      :class="[
        'group/item relative flex flex-col gap-1 rounded-md bg-card/20 p-2 text-left',
        'cursor-pointer transition-colors select-none',
        isDragging ? 'cursor-grabbing opacity-50 ring-1 ring-primary' : '',
        isSelected ? 'bg-selection text-selection-foreground' : email.is_read ? '' : 'text-primary',
      ]"
      @click="emit('click')"
    >
      <div class="flex items-center gap-1.5 text-sm">
        <div
          v-if="!email.is_read"
          class="h-2 w-2 rounded-full bg-accent"
        />
        <RavnAvatar
          :email="email.from.address"
          :name="email.from.name"
          class="pointer-events-none shrink-0"
          size="xs"
        />

        <span class="min-w-0 flex-1 truncate leading-tight font-medium">
          {{ email.from.name || email.from.address }}
        </span>
        <span class="ml-auto text-sm text-nowrap opacity-60">{{ timeLabel }}</span>
      </div>

      <div class="flex items-start gap-1">
        <span class="line-clamp-1 font-bold">
          {{ email.subject || t('components.emailViewer.noSubject') }}
        </span>

        <Icon
          v-if="hasReminder"
          class="shrink-0 text-primary"
          name="lucide:bell"
        />
        <Icon
          v-if="email.has_attachments"
          class="shrink-0 opacity-50"
          name="lucide:paperclip"
        />
      </div>

      <div class="line-clamp-2 text-sm opacity-50">{{ email.snippet }}</div>

      <div
        v-if="email.labels?.length"
        class="flex flex-wrap items-center gap-1 pt-0.5"
      >
        <EmailLabel
          v-for="label in email.labels"
          :key="label.id"
          v-bind="label"
        />
      </div>
    </div>
  </MailContextMenu>
</template>
