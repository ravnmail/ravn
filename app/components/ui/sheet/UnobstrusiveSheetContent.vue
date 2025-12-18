<script lang="ts" setup>
import type { DialogContentEmits, DialogContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import {
  type SheetVariants,
  UnobstrusiveSheetContent,
  unobstrusiveSheetContentVariants,
  type UnobstrusiveSheetVariants
} from '.'
import { cn } from '@/lib/utils'
import { unobstrusiveSheetVariants } from '.'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '~/components/ui/resizable'


interface SheetContentProps extends DialogContentProps {
  class?: HTMLAttributes['class']
  side?: UnobstrusiveSheetVariants
}

defineOptions({
  inheritAttrs: false,
})

const props = defineProps<SheetContentProps>()

const emits = defineEmits<DialogContentEmits>()

</script>

<template>
  <ResizablePanelGroup
    :class="cn(unobstrusiveSheetVariants({ side }), props.class)"
    direction="horizontal"
  >
    <ResizablePanel class="pointer-events-none"/>
    <ResizableHandle/>
    <ResizablePanel
      :class="cn(unobstrusiveSheetContentVariants({ side }))"
      :min-size="480"
      size-unit="px"
    >
      <div class="relative flex flex-1">
        <slot/>
        <button
          class="flex items-center absolute -left-2 top-1 rounded-lg p-1.5 ring-offset-background transition-all hover:bg-button-ghost-hover/50 hover:text-primary focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted"
          @click="emits('close', false)"
        >
          <Icon
            :size="18"
            name="lucide:chevrons-right"
          />
        </button>
      </div>
    </ResizablePanel>
  </ResizablePanelGroup>
</template>
