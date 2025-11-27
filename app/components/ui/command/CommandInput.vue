<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { ListboxFilter, type ListboxFilterProps, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'
import { useCommand } from '.'

defineOptions({
  inheritAttrs: false,
})

const props = defineProps<ListboxFilterProps & {
  class?: HTMLAttributes['class']
}>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)

const { filterState } = useCommand()
</script>

<template>
  <div
    class="flex items-center border-b border-border px-3 gap-2"
    cmdk-input-wrapper
  >
    <Icon
      class="shrink-0"
      name="lucide:search"
    />
    <ListboxFilter
      v-model="filterState.search"
      :class="cn('flex w-full rounded-md bg-transparent py-3 outline-none placeholder:text-muted disabled:cursor-not-allowed disabled:opacity-50', props.class)"
      auto-focus
      v-bind="{ ...forwardedProps, ...$attrs }"
    />
  </div>
</template>
