<script lang="ts" setup>
import ReminderIndicator from '~/components/Ravn/ReminderIndicator.vue'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import useFormatting from '~/composables/useFormatting'
import type { EmailCategory, EmailListItem } from '~/types/email'

interface Props extends EmailListItem {
  leftActions?: SwipeAction[]
  rightActions?: SwipeAction[]
  excludeLabels?: string[]
  message_count?: number
  isSelected?: boolean
  isMultiSelected?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  leftActions: () => [],
  rightActions: () => [],
  excludeLabels: () => [],
  message_count: 0,
  isSelected: false,
  isMultiSelected: false,
})

const emit = defineEmits<{
  (e: 'action', actionId: string, emailId: string): void
}>()

const { formatEmailDate } = useFormatting()

const itemRef = ref<HTMLElement | null>(null)

const mappedLeftActions = computed(() =>
  props.leftActions.map((action) => ({
    ...action,
    handler: () => emit('action', action.id, props.id),
  }))
)

const mappedRightActions = computed(() =>
  props.rightActions.map((action) => ({
    ...action,
    handler: () => emit('action', action.id, props.id),
  }))
)

const { swipeOffset, activeActionSide, visibleSide, activeActionIndex, handleActionClick } =
  useSwipeActions(itemRef, {
    leftActions: mappedLeftActions.value,
    rightActions: mappedRightActions.value,
  })

const contentTransform = computed(() => `translateX(${swipeOffset.value}px)`)

const actionWidth = 5
const leftActionsWidth = computed(() => props.leftActions.length * actionWidth)
const rightActionsWidth = computed(() => props.rightActions.length * actionWidth)

const filteredLabels = computed(() => {
  if (props.excludeLabels && props.excludeLabels.length > 0) {
    return props.labels.filter((l) => !props.excludeLabels!.includes(l.id))
  }
  return props.labels
})

const categoryIconMap: Record<EmailCategory, { name: string; color: string }> = {
  personal: {
    name: 'lucide:user',
    color: '#3b82f6',
  },
  promotions: {
    name: 'lucide:tag',
    color: '#4caf50',
  },
  updates: {
    name: 'lucide:megaphone',
    color: '#ff9800',
  },
  transactions: {
    name: 'lucide:shopping-cart',
    color: '#f44336',
  },
}
</script>

<template>
  <div
    ref="itemRef"
    class="relative touch-pan-x overflow-hidden rounded"
  >
    <div
      v-if="props.leftActions.length > 0"
      :aria-hidden="visibleSide !== 'left'"
      :class="['absolute inset-y-0 left-0 flex h-full', visibleSide !== 'left' && 'hidden']"
      :style="{ width: `${leftActionsWidth}rem` }"
    >
      <RavnSwipeActionItem
        v-for="(action, index) in props.leftActions"
        :key="action.id"
        :action="action"
        :is-active="visibleSide === 'left' && activeActionIndex === index"
        side="left"
        @click="handleActionClick(index, 'left')"
      />
    </div>
    <div
      v-if="rightActions.length > 0"
      :aria-hidden="visibleSide !== 'right'"
      :class="['absolute inset-y-0 right-0 flex h-full', visibleSide !== 'right' && 'hidden']"
      :style="{ width: `${rightActionsWidth}rem` }"
    >
      <RavnSwipeActionItem
        v-for="(action, index) in rightActions"
        :key="action.id"
        :action="action"
        :is-active="visibleSide === 'right' && activeActionIndex === index"
        side="right"
        @click="handleActionClick(index, 'right')"
      />
    </div>

    <div :style="{ transform: contentTransform }">
      <div
        :class="[
          'flex p-2 transition-transform duration-200 ease-out',
          is_read ? '' : 'text-primary',
          isSelected || isMultiSelected ? 'bg-selection text-selection-foreground' : '',
        ]"
      >
        <RavnAvatar
          v-if="from.address"
          :email="from.address"
          :name="from.name"
          class="pointer-events-none mr-4"
          size="lg"
        />
        <div class="flex-grow">
          <div class="flex items-center gap-2">
            <div
              v-if="!is_read"
              class="h-2 w-2 rounded-full bg-accent"
            />
            <span class="line-clamp-1 text-sm">{{ from.name || from.address }}</span>
            <span class="ml-auto text-sm text-nowrap opacity-60">{{
              formatEmailDate($props, 2)
            }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="line-clamp-1 font-bold">{{ subject }}</span>
            <div class="ml-auto flex items-center gap-2">
              <Icon
                v-if="has_attachments"
                class="shrink-0"
                name="lucide:paperclip"
              />
              <ReminderIndicator
                :notified-at="notified_at"
                :remind-at="remind_at"
              />
              <Icon
                v-if="category"
                :name="categoryIconMap[category].name"
                :style="{ color: categoryIconMap[category].color }"
                class="shrink-0"
              />
              <span
                v-if="message_count"
                class="rounded px-2 py-0.5 text-xs font-bold"
                >{{ message_count }}</span
              >
            </div>
          </div>
          <div class="line-clamp-2 text-sm opacity-50">{{ snippet }}</div>
          <div class="mt-2 flex flex-wrap gap-1">
            <EmailLabel
              v-for="l in filteredLabels"
              :key="l.id"
              v-bind="l"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
