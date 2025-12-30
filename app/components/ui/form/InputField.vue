<script lang="ts" setup>
import type { HTMLAttributes } from 'vue'
import { useVModel } from '@vueuse/core'
import FormField from './FormField.vue'
import Input from '../input/Input.vue'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import { Button } from '~/components/ui/button'
import { toast } from 'vue-sonner'

type InputActionType = 'clear' | 'copy'

const { $t } = useI18n()

const props = defineProps<{
  // FormField props
  id?: string
  label?: string | CleanTranslation
  required?: boolean
  tooltip?: string | CleanTranslation
  description?: string | CleanTranslation
  error?: string
  autoFocus?: boolean
  icon?: string
  class?: HTMLAttributes['class']
  inputClass?: HTMLAttributes['class']
  actions?: Array<InputActionType>

  // Input props
  modelValue?: string | number
  defaultValue?: string | number
  placeholder?: unknown
  type?: string
  disabled?: boolean
  readonly?: boolean
  name: string
}>()

const icons = {
  clear: 'lucide:x',
  copy: 'lucide:copy',
}

const emits = defineEmits<{
  (e: 'update:modelValue' | InputActionType, payload: string | number): void
}>()

// Create two-way binding for modelValue
const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
  defaultValue: props.defaultValue,
})

// Additional input props that should be passed to the Input component
const inputProps = computed(() => {
  const {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    id, label, tooltip, description, error, class: _class,
    modelValue: _modelValue, defaultValue: _defaultValue,
    ...rest
  } = props

  return { ...rest, class: props.inputClass }
})

const trigger = (action: InputActionType) => {
  if (action === 'clear') {
    modelValue.value = ''
  } else if (action === 'copy') {
    navigator.clipboard.writeText(modelValue.value as string)
    toast.info($t('notifications.inputField.copied'))
  }
}

</script>

<template>
  <FormField
    v-slot="{ id: uniqueId, hasError }"
    :class="props.class"
    :description="description"
    :error="error"
    :label="label"
    :name="name"
    :required="required"
    :tooltip="tooltip"
  >
    <div class="relative">
      <Icon
        v-if="icon"
        :name="icon"
        class="absolute left-2 top-1/2 -translate-y-1/2 text-muted-foreground"
      />
      <Input
        :id="uniqueId"
        v-model="modelValue"
        :class="{ 'border-red-500': hasError, 'pl-8': icon, 'pr-10': actions?.length }"
        v-bind="{ ...inputProps, ...$attrs }"
      />
      <div
        v-if="actions?.length"
        class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-0.5"
      >
        <Button
          v-for="action in actions"
          :key="action"
          :aria-label="action"
          size="bar"
          @click="trigger(action)"
        >
          <Icon :name="icons[action]" />
        </Button>
      </div>
    </div>
  </FormField>
</template>