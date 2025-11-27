<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { TagsInputRoot, type TagsInputRootEmits, type TagsInputRootProps, useForwardPropsEmits } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<TagsInputRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<TagsInputRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <TagsInputRoot
    :class="cn('flex h-9 flex-wrap gap-2 items-center rounded-md border border-input bg-input px-3 py-1 text-sm shadow-sm', props.class)"
    v-bind="forwarded"
  >
    <slot/>
  </TagsInputRoot>
</template>
