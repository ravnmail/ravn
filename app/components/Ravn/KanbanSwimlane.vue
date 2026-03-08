<script lang="ts" setup>
import KanbanEmailItem from '~/components/Ravn/KanbanEmailItem.vue'
import MailContextMenu from '~/components/Ravn/MailContextMenu.vue'
import { Badge } from '~/components/ui/badge'
import { Button } from '~/components/ui/button'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import IconName from '~/components/ui/IconName.vue'
import IconNameField from '~/components/ui/IconNameField.vue'
import { Popover, PopoverAnchor, PopoverContent } from '~/components/ui/popover'
import { SimpleTooltip } from '~/components/ui/tooltip'
import type { DragData } from '~/composables/useDragAndDrop'
import { useDropTarget } from '~/composables/useDragAndDrop'
import type { EmailListItem } from '~/types/email'
import type { KanbanSwimlane } from '~/types/view'

const props = defineProps<{
  swimlane: KanbanSwimlane
  emails: EmailListItem[]
  selectedConversationId?: string
  selectedEmailIds?: string[]
}>()

const editValue = ref<{ icon?: string | null; name: string; color?: string | null } | null>(null)

const emit = defineEmits<{
  (e: 'update', value: KanbanSwimlane): void
  (e: 'drop', dragData: DragData, targetSwimlaneId: string): void
  (e: 'emailClick', email: EmailListItem, event: MouseEvent): void
  (e: 'refresh'): void
}>()

const { t } = useI18n()
const swimlaneRef = ref<HTMLElement | null>(null)

const { isOver, canDrop } = useDropTarget(swimlaneRef, {
  getData: () => ({
    type: 'swimlane',
    id: props.swimlane.id,
  }),
  canDrop: (data: DragData) => {
    return data.type === 'email' && data.fromSwimlaneId !== props.swimlane.id
  },
  onDrop: async (data: DragData) => {
    emit('drop', data, props.swimlane.id)
  },
})

const dropStateClass = computed(() => {
  if (isOver.value && canDrop.value) {
    return 'bg-primary/10 ring-2 ring-primary ring-offset-1'
  }

  if (isOver.value && !canDrop.value) {
    return 'ring-2 ring-destructive ring-offset-1'
  }

  return ''
})

const swimlaneBaseStyle = computed(() => ({
  backgroundColor: props.swimlane.color ? `${props.swimlane.color}15` : 'transparent',
}))

const backgroundColor = computed(() =>
  props.swimlane.color ? `${props.swimlane.color}15` : 'transparent'
)

const collapsed = computed(() => props.swimlane.state === 'closed')

const setCollapsed = (value: boolean) => {
  emit('update', {
    ...props.swimlane,
    state: value ? 'closed' : 'open',
  })
}

const startEdit = () => {
  editValue.value = {
    icon: props.swimlane.icon,
    color: props.swimlane.color,
    name: props.swimlane.title,
  }
}
const cancelEdit = () => {
  editValue.value = null
}

const isEditing = computed({
  get: () => editValue.value !== null,
  set: (v: boolean) => {
    if (!v) {
      cancelEdit()
    } else {
      startEdit()
    }
  },
})

const handleRename = () => {
  if (editValue.value) {
    emit('update', {
      ...props.swimlane,
      icon: editValue.value.icon ?? undefined,
      color: editValue.value.color ?? undefined,
      title: editValue.value.name,
    })
    cancelEdit()
  }
}

// Context menu — one instance per lane, tracks the right-clicked email
const { archive, trash, move, updateRead, addLabelToEmail, removeLabelFromEmail, setRemindAt } =
  useEmails()
const contextEmail = ref<EmailListItem | null>(null)

const replaceContextEmail = (updater: (email: EmailListItem) => EmailListItem) => {
  if (!contextEmail.value) return
  contextEmail.value = updater(contextEmail.value)
}

const toggleLabelOptimistically = async (email: EmailListItem, labelId: string) => {
  const existingLabels = email.labels || []
  const hasLabel = existingLabels.some((label) => label.id === labelId)
  const previousEmail = { ...email, labels: [...existingLabels] }

  if (hasLabel) {
    replaceContextEmail((currentEmail) => ({
      ...currentEmail,
      labels: currentEmail.labels.filter((label) => label.id !== labelId),
    }))

    try {
      await removeLabelFromEmail(email.id, labelId)
    } catch (error) {
      contextEmail.value = previousEmail
      throw error
    }

    return
  }

  const labelToAdd = existingLabels.find((label) => label.id === labelId)

  replaceContextEmail((currentEmail) => ({
    ...currentEmail,
    labels: labelToAdd ? [...currentEmail.labels, labelToAdd] : currentEmail.labels,
  }))

  try {
    await addLabelToEmail({ email_id: email.id, label_id: labelId })
  } catch (error) {
    contextEmail.value = previousEmail
    throw error
  }
}

const getEffectiveSelectedEmailIds = () => {
  const selectedIds = props.selectedEmailIds || []
  if (contextEmail.value && selectedIds.includes(contextEmail.value.id)) {
    return selectedIds
  }
  return contextEmail.value ? [contextEmail.value.id] : selectedIds
}

const executeAction = async (actionId: string, arg?: unknown) => {
  const email = contextEmail.value
  const targetEmailIds = getEffectiveSelectedEmailIds()

  if (!email || targetEmailIds.length === 0) return

  switch (actionId) {
    case 'archiveEmail':
      await Promise.all(targetEmailIds.map((emailId) => archive(emailId)))
      emit('refresh')
      break
    case 'deleteEmail':
      await Promise.all(targetEmailIds.map((emailId) => trash(emailId)))
      emit('refresh')
      break
    case 'moveEmail':
      await Promise.all(targetEmailIds.map((emailId) => move(emailId, arg as string)))
      emit('refresh')
      break
    case 'markRead':
      await Promise.all(targetEmailIds.map((emailId) => updateRead(emailId, true)))
      if (targetEmailIds.includes(email.id)) {
        replaceContextEmail((currentEmail) => ({
          ...currentEmail,
          is_read: true,
        }))
      }
      break
    case 'markUnread':
      await Promise.all(targetEmailIds.map((emailId) => updateRead(emailId, false)))
      if (targetEmailIds.includes(email.id)) {
        replaceContextEmail((currentEmail) => ({
          ...currentEmail,
          is_read: false,
        }))
      }
      break
    case 'assignLabel':
      if (typeof arg === 'string') {
        await Promise.all(
          targetEmailIds.map((emailId) => addLabelToEmail({ email_id: emailId, label_id: arg }))
        )
        emit('refresh')
      }
      break
    case 'removeLabel':
      if (typeof arg === 'string') {
        await Promise.all(targetEmailIds.map((emailId) => removeLabelFromEmail(emailId, arg)))
        emit('refresh')
      }
      break
    case 'setRemindAt':
      await Promise.all(targetEmailIds.map((emailId) => setRemindAt(emailId, arg as string | null)))
      if (targetEmailIds.includes(email.id)) {
        replaceContextEmail((currentEmail) => ({
          ...currentEmail,
          remind_at: (arg as string | null) ?? undefined,
        }))
      }
      emit('refresh')
      break
  }
}
</script>

<template>
  <!-- Collapsed state: narrow vertical strip, full height -->
  <div
    v-if="collapsed"
    ref="swimlaneRef"
    class="group/swimlane w-12 shrink-0 self-stretch"
  >
    <div
      :class="[
        'relative flex h-full w-full cursor-pointer flex-col items-center justify-start rounded-lg py-3 transition-colors duration-200 hover:bg-white/5',
        dropStateClass,
      ]"
      :style="swimlaneBaseStyle"
      @click="setCollapsed(false)"
    >
      <div class="flex flex-1 -rotate-90 items-center gap-2">
        <IconName
          :color="swimlane.color"
          :icon="swimlane.icon || 'folder-open'"
          :name="swimlane.title"
          class="w-fit whitespace-nowrap"
        />
        <Badge
          size="sm"
          variant="background"
          class="shrink-0"
        >
          {{ emails.length > 99 ? '+99' : emails.length }}
        </Badge>
      </div>
      <div class="mt-3 shrink-0">
        <Icon
          name="lucide:chevron-right"
          class="text-foreground/40 transition-colors group-hover/swimlane:text-foreground/70"
        />
      </div>
    </div>
  </div>

  <!-- Expanded state -->
  <div
    v-else
    ref="swimlaneRef"
    :class="['group/swimlane flex w-80 flex-shrink-0 flex-col rounded-lg', dropStateClass]"
    :style="swimlaneBaseStyle"
  >
    <!-- Header -->
    <div class="mb-3 flex flex-col gap-2">
      <Popover
        :open="isEditing"
        @update:open="
          (v: boolean) => {
            isEditing = v
          }
        "
      >
        <PopoverAnchor />
        <PopoverContent
          :align-offset="8"
          :side-offset="28"
          align="start"
          class="flex items-center gap-1"
          side="bottom"
        >
          <IconNameField
            v-model="editValue"
            name="icon"
            @cancel="cancelEdit"
            @submit="handleRename"
          />
          <Button
            size="xs"
            variant="ghost"
            @click="handleRename"
          >
            <Icon
              class="text-success"
              name="lucide:check"
            />
          </Button>
          <SimpleTooltip tooltip-markdown="press `ESC` to cancel">
            <Button
              size="xs"
              variant="ghost"
              @click="cancelEdit"
            >
              <Icon
                class="text-destructive"
                name="lucide:x"
              />
            </Button>
          </SimpleTooltip>
        </PopoverContent>
      </Popover>

      <div class="flex items-center">
        <div class="flex flex-1 items-center gap-2">
          <IconName
            :color="swimlane.color"
            :icon="swimlane.icon || 'folder-open'"
            :name="swimlane.title"
            @dblclick="isEditing = true"
          />
          <SimpleTooltip :tooltip-markdown="t('components.kanban.swimlane.actions.collapse')">
            <Button
              class="opacity-0 transition-opacity group-hover/swimlane:opacity-100"
              size="xs"
              variant="ghost"
              @click="setCollapsed(true)"
            >
              <Icon name="lucide:chevron-left" />
            </Button>
          </SimpleTooltip>
        </div>
        <Badge
          size="sm"
          variant="background"
        >
          {{ emails.length > 99 ? '+99' : emails.length }}
        </Badge>
      </div>

      <div
        :style="{ backgroundColor: swimlane.color }"
        class="h-1 w-full rounded-full"
      />
    </div>

    <!-- Email list -->
    <div
      :style="{ backgroundColor }"
      class="min-h-0 flex-1 rounded-lg p-2 transition-all duration-200"
    >
      <MailContextMenu
        :selected-email-ids="getEffectiveSelectedEmailIds()"
        :active-email="contextEmail"
        :on-execute-action="executeAction"
      >
        <div>
          <KanbanEmailItem
            v-for="email in emails"
            :key="email.id"
            :email="email"
            :exclude-labels="swimlane.label_ids"
            :is-selected="(selectedEmailIds || []).includes(email.id)"
            :is-multi-selected="false"
            :selected-ids="selectedEmailIds || []"
            :swimlane-id="swimlane.id"
            @contextmenu.capture="contextEmail = email"
            @click="emit('emailClick', email, $event)"
          />
        </div>
      </MailContextMenu>
      <EmptyState
        v-if="emails.length === 0"
        :style="{ color: swimlane.color || undefined }"
        :title="t('components.kanban.emptyState.noEmails')"
        class="opacity-30"
        icon-name="lucide:folder-open"
        variant="coloredSticker"
      />
    </div>
  </div>
</template>
