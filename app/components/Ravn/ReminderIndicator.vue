<script lang="ts" setup>
import dayjs from 'dayjs'
import LocalizedFormat from 'dayjs/plugin/localizedFormat'

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '~/components/ui/tooltip'

dayjs.extend(LocalizedFormat)

const props = withDefaults(
  defineProps<{
    remindAt?: string | null
    notifiedAt?: string | null
    class?: string
  }>(),
  {
    remindAt: null,
    notifiedAt: null,
    class: undefined,
  }
)

const parsedRemindAt = computed(() => (props.remindAt ? dayjs(props.remindAt) : null))
const parsedNotifiedAt = computed(() => (props.notifiedAt ? dayjs(props.notifiedAt) : null))

const hasReminder = computed(() => !!props.remindAt && !!parsedRemindAt.value?.isValid())

const hasBeenNotified = computed(() => {
  if (!parsedRemindAt.value?.isValid() || !parsedNotifiedAt.value?.isValid()) {
    return false
  }

  return (
    parsedNotifiedAt.value.isAfter(parsedRemindAt.value) ||
    parsedNotifiedAt.value.isSame(parsedRemindAt.value)
  )
})

const tooltipLabel = computed(() => {
  if (!props.remindAt) return ''
  if (!parsedRemindAt.value?.isValid()) return props.remindAt
  return parsedRemindAt.value.format('LLLL')
})

const iconName = computed(() => (hasBeenNotified.value ? 'lucide:bell-ring' : 'lucide:bell'))
</script>

<template>
  <TooltipProvider v-if="hasReminder">
    <Tooltip :delay-duration="0">
      <TooltipTrigger>
        <Icon
          :class="['shrink-0', hasBeenNotified ? 'text-muted' : 'text-primary', props.class]"
          :name="iconName"
        />
      </TooltipTrigger>
      <TooltipContent>
        {{ tooltipLabel }}
      </TooltipContent>
    </Tooltip>
  </TooltipProvider>
</template>
