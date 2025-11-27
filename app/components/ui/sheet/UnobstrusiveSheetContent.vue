<script lang="ts" setup>
import type { DialogContentEmits, DialogContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import type { SheetVariants } from '.'
import { cn } from '@/lib/utils'
import { sheetVariants } from '.'
import { Button } from '~/components/ui/button'

interface SheetContentProps extends DialogContentProps {
  class?: HTMLAttributes['class']
  side?: SheetVariants['side']
}

defineOptions({
  inheritAttrs: false,
})

const props = defineProps<SheetContentProps>()

const emits = defineEmits<DialogContentEmits>()

</script>

<template>
  <div
    :class="cn(sheetVariants({ side }), props.class)"
  >
    <slot/>

    <Button
      class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-secondary"
      @click="emits('close', false)"
    >
      <Icon name="lucide:x"/>
    </Button>
  </div>
</template>
