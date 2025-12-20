<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { TagsInputItemDelete, type TagsInputItemDeleteProps, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<TagsInputItemDeleteProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <TagsInputItemDelete
    :class="cn('flex rounded bg-transparent mr-1 hover:text-destructive', props.class)"
    v-bind="forwardedProps"
  >
    <slot>
      <Icon name="lucide:x"/>
    </slot>
  </TagsInputItemDelete>
</template>
