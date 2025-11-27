<script setup lang="ts">
import { cn } from '@/lib/utils'
import { AlertDialogAction, type AlertDialogActionProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<AlertDialogActionProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})
</script>

<template>
  <AlertDialogAction
    v-bind="delegatedProps"
    :class="cn(props.class)"
  >
    <slot/>
  </AlertDialogAction>
</template>
