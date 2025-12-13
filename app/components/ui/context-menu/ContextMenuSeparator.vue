<script lang="ts" setup>
import type { ContextMenuSeparatorProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  ContextMenuSeparator,
} from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<ContextMenuSeparatorProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <ContextMenuSeparator
    :class="cn('mx-2 my-1 h-px bg-popover-border', props.class)"
    v-bind="delegatedProps"
  />
</template>
