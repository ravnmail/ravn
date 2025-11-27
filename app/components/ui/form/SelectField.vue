<script generic="T" lang="ts" setup>
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { useVModel } from '@vueuse/core'
import FormField from './FormField.vue'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue
} from '~/components/ui/select'
import type { CleanTranslation } from 'nuxt-i18n-micro-types/src'

export interface SelectOption<T = unknown> {
  value: T
  label: string
  disabled?: boolean
}

const props = defineProps<{
  id?: string
  label?: string | CleanTranslation
  required?: boolean
  tooltip?: string | CleanTranslation
  description?: string | CleanTranslation
  error?: string
  class?: HTMLAttributes['class']
  selectClass?: HTMLAttributes['class']
  modelValue?: T
  defaultValue?: T
  placeholder?: string | CleanTranslation
  disabled?: boolean
  readonly?: boolean
  name: string
  options: SelectOption<T>[]
  displayFn?: (option: SelectOption<T>) => string
  valueFn?: (option: SelectOption<T>) => T
  emptyText?: string | CleanTranslation
}>()

const emits = defineEmits<{
  (e: 'update:modelValue', payload: T): void
  (e: 'select' | 'remove', payload: { option: SelectOption<T>, value: T }): void
}>()

const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
  defaultValue: props.defaultValue,
})

const getDisplayValue = (option: SelectOption<T>): string => {
  return props.displayFn ? props.displayFn(option) : option.label
}

const getOptionValue = (option: SelectOption<T>): T => {
  return props.valueFn ? props.valueFn(option) : option.value
}

const getOptionByValue = (value: T): SelectOption<T> | undefined => {
  return props.options.find(option => {
    const optionValue = getOptionValue(option)
    return optionValue === value
  })
}

const emptyTextComputed = computed(() => {
  return props.emptyText || 'common.no_results'
})

const handleSelect = (value: T) => {
  modelValue.value = value
  const option = getOptionByValue(value)
  if (option) {
    emits('select', { option, value })
  }
}
</script>

<template>
  <FormField
    :id="id"
    :class="props.class"
    :description="description"
    :error="error"
    :label="label"
    :name="name"
    :required="required"
    :tooltip="tooltip"
  >
    <template #default="{ id, hasError }">
      <Select
        :class="[selectClass, { 'opacity-50': disabled || readonly }]"
        :disabled="disabled || readonly"
        :model-value="modelValue"
        @update:model-value="handleSelect"
      >
        <SelectTrigger
          :id="id"
          :class="{ 'border-destructive': hasError }"
        >
          <SelectValue :placeholder="$t(String(placeholder || 'common.select'))">
            {{ getOptionByValue(modelValue)?.label || '' }}
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <template v-if="options.length">
              <SelectItem
                v-for="option in options"
                :key="String(getOptionValue(option))"
                :disabled="option.disabled"
                :value="getOptionValue(option)"
              >
                {{ getDisplayValue(option) }}
              </SelectItem>
            </template>
            <template v-else>
              <SelectLabel class="opacity-50">{{ $t(String(emptyTextComputed)) }}</SelectLabel>
            </template>
          </SelectGroup>
        </SelectContent>
      </Select>
    </template>
  </FormField>
</template>

