<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { TagsInputItem, type TagsInputItemProps, useForwardProps } from 'reka-ui'

import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<TagsInputItemProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <TagsInputItem
    :class="cn('flex items-center rounded-md bg-popover data-[state=active]:ring-ring data-[state=active]:ring-2 data-[state=active]:ring-offset-2 ring-offset-background', props.class)"
    v-bind="forwardedProps"
  >
    <slot/>
  </TagsInputItem>
</template>
