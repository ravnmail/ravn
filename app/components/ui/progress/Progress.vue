<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { ProgressIndicator, ProgressRoot, type ProgressRootProps, } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = withDefaults(
  defineProps<ProgressRootProps & { class?: HTMLAttributes['class'] }>(),
  {
    modelValue: 0,
  },
)

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})
</script>

<template>
  <ProgressRoot
    :class="
      cn(
        'relative h-1.5 w-full overflow-hidden rounded-full bg-muted/30',
        props.class,
      )
    "
    v-bind="delegatedProps"
  >
    <ProgressIndicator
      :style="`transform: translateX(-${100 - (props.modelValue ?? 0)}%);`"
      class="h-full w-full flex-1 bg-primary transition-all rounded-full"
    />
  </ProgressRoot>
</template>
