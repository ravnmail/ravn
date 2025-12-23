<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { SelectGroup, type SelectGroupProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<SelectGroupProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})
</script>

<template>
  <SelectGroup
    :class="cn('w-full', props.class)"
    v-bind="delegatedProps"
  >
    <slot/>
  </SelectGroup>
</template>
