<script lang="ts" setup>
import { cva, type VariantProps } from 'class-variance-authority'
import type { HTMLAttributes } from 'vue'
import { cn } from '~/lib/utils'

const alertVariants = cva('relative w-full rounded-lg border p-4', {
  variants: {
    variant: {
      default: 'border-border bg-background text-foreground',
      ai: 'border-ai-border text-ai bg-ai-background/10',
      info: 'border-info-border text-info bg-info-background/10',
      destructive: 'border-destructive-border text-destructive bg-destructive-background/10',
      success: 'border-success-border text-success bg-success-background/10',
      warning: 'border-warning-border text-warning bg-warning-background/10',
    },
  },
  defaultVariants: {
    variant: 'default',
  },
})

type AlertVariants = VariantProps<typeof alertVariants>

interface AlertProps {
  variant?: AlertVariants['variant']
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<AlertProps>(), {
  variant: 'default',
})
</script>

<template>
  <div
    :class="cn(alertVariants({ variant }), props.class)"
    role="alert"
  >
    <slot />
  </div>
</template>
