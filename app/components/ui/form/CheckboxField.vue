<script lang="ts" setup>

import type { HTMLAttributes } from 'vue'
import { useVModel } from '@vueuse/core'
import type { Translation } from 'nuxt-i18n-micro-types/src'
import { Checkbox } from '~/components/ui/checkbox'
import { TooltipIcon } from '~/components/ui/tooltip'

const props = defineProps<{
  // FormField props
  id?: string
  label: string | Translation
  required?: boolean
  tooltip?: string | Translation
  description?: string | Translation
  error?: string
  class?: HTMLAttributes['class']

  // Input props
  modelValue?: boolean
  defaultValue?: boolean
  type?: string
  disabled?: boolean
  readonly?: boolean
  name: string
}>()

const emits = defineEmits<{
  (e: 'update:modelValue', payload: string | number): void
}>()

// Create two-way binding for modelValue
const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
  defaultValue: props.defaultValue,
})

const uniqueId = computed(() => props.id || `${props.name}-${Math.random().toString(36).substring(2, 9)}`)
const hasError = computed(() => !!props.error)

</script>

<template>
  <div class="grid w-full gap-2">
    <div class="flex gap-3 items-center">
      <Checkbox
        :id="uniqueId"
        v-model="modelValue"
        :class="[hasError && 'border border-destructive']"
        name="translatable"
      />
      <div class="flex gap-2 items-center">
        <label
          :for="uniqueId"
          class="font-semibold text-primary"
        >
          {{ label }}
        </label>
        <TooltipIcon
          v-if="tooltip"
          size="1.2em"
        >{{ tooltip }}</TooltipIcon>
      </div>
    </div>
    <p
      v-if="hasError"
      class="mt-1 whitespace-pre-line text-sm leading-tight text-destructive"
    >
      {{ error }}
    </p>
    <p
      v-if="description"
      class="text-sm text-text-muted"
    >
      {{ description }}
    </p>
  </div>
</template>