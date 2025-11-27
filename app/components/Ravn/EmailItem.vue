<script lang="ts" setup>

import type { EmailListItem } from '~/types/email'
import useFormatting from '~/composables/useFormatting'
import EmailLabel from '~/components/ui/EmailLabel.vue'

interface Props extends EmailListItem {
  leftActions?: SwipeAction[]
  rightActions?: SwipeAction[]
  excludeLabels?: string[]
  message_count?: number
  isSelected?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  leftActions: () => [],
  rightActions: () => [],
  excludeLabels: () => [],
  message_count: 0,
  isSelected: false
})

const emit = defineEmits<{
  (e: 'action', actionId: string, emailId: string): void
}>()

const { formatEmailDate } = useFormatting()

const itemRef = ref<HTMLElement | null>(null)


const mappedLeftActions = computed(() =>
  props.leftActions.map(action => ({
    ...action,
    handler: () => emit('action', action.id, props.id)
  }))
)

const mappedRightActions = computed(() =>
  props.rightActions.map(action => ({
    ...action,
    handler: () => emit('action', action.id, props.id)
  }))
)

const {
  swipeOffset,
  activeActionSide,
  visibleSide,
  activeActionIndex,
  handleActionClick
} = useSwipeActions(itemRef, {
  leftActions: mappedLeftActions.value,
  rightActions: mappedRightActions.value
})

const contentTransform = computed(() => `translateX(${swipeOffset.value}px)`)

const actionWidth = 5
const leftActionsWidth = computed(() => props.leftActions.length * actionWidth)
const rightActionsWidth = computed(() => props.rightActions.length * actionWidth)

const filteredLabels = computed(() => {
  if (props.excludeLabels && props.excludeLabels.length > 0) {
    return props.labels.filter(l => !props.excludeLabels!.includes(l.id))
  }
  return props.labels
})

</script>

<template>
  <div
    ref="itemRef"
    class="relative overflow-hidden touch-pan-x rounded"
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
        :class="['flex p-2 transition-transform duration-200 ease-out', is_read ? '' : 'text-primary', isSelected ? 'bg-selection text-selection-foreground' : '']"
      >
        <RavnAvatar
          v-if="from.address"
          :account-id="account_id"
          :email="from.address"
          :name="from.name"
          class="mr-4 pointer-events-none"
          size="lg"
        />
        <div class="flex-grow">
          <div class="flex items-center gap-2">
            <div
              v-if="!is_read"
              class="w-2 h-2 bg-accent rounded-full"
            />
            <span class="line-clamp-1 text-sm">{{ from.name || from.address }}</span>
            <span class="ml-auto text-sm opacity-60 text-nowrap">{{ formatEmailDate($props, 2) }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="font-bold line-clamp-1">{{ subject }}</span>
            <div class="ml-auto flex items-center gap-2">
              <Icon
                v-if="has_attachments"
                class="shrink-0 opacity-50"
                name="lucide:paperclip"
              />
              <span
                v-if="message_count"
                class="text-xs font-bold px-2 py-0.5 rounded"
              >{{ message_count }}</span>
            </div>
          </div>
          <div class="text-sm opacity-50 line-clamp-2">{{ snippet }}</div>
          <div class="flex mt-2 gap-1 flex-wrap">
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
