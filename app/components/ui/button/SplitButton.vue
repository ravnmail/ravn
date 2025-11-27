<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { type ButtonVariants, buttonVariants } from './index'
import { Primitive, type PrimitiveProps } from 'reka-ui'
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '../dropdown-menu'

interface Props extends PrimitiveProps {
  variant?: ButtonVariants['variant']
  size?: ButtonVariants['size']
  class?: HTMLAttributes['class']
  primaryAction?: () => void
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  as: 'div',
  variant: 'default',
  size: 'default',
  disabled: false
})

const primaryButtonClasses = computed(() => {
  return [
    buttonVariants({ variant: props.variant, size: props.size }),
    'rounded-r-none',
    props.class
  ].filter(Boolean)
})

const triggerButtonClasses = computed(() => {
  return [
    buttonVariants({ variant: props.variant, size: props.size }),
    'rounded-l-none border-l border-l-surface !px-2'
  ].filter(Boolean)
})
</script>

<template>
  <Primitive
    :as="as"
    :as-child="asChild"
    class="flex"
  >
    <DropdownMenu>
      <button
        :class="primaryButtonClasses"
        :disabled="disabled"
        @click="primaryAction && primaryAction()"
      >
        <slot/>
      </button>
      <DropdownMenuTrigger
        :disabled="disabled"
        :class="triggerButtonClasses"
      >
        <Icon name="lucide:chevron-down"/>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <slot name="menu"/>
      </DropdownMenuContent>
    </DropdownMenu>
  </Primitive>
</template>