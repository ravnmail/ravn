<script lang="ts" setup>
import type { HTMLAttributes } from 'vue'
import { useVModel } from '@vueuse/core'
import FormField from './FormField.vue'
import { Textarea } from '~/components/ui/textarea'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import { Button } from '~/components/ui/button'
import { SimpleTooltip } from '~/components/ui/tooltip'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'

const props = defineProps<{
  // FormField props
  id?: string
  label?: string | CleanTranslation
  fullLabel?: string | CleanTranslation
  required?: boolean
  tooltip?: string | CleanTranslation
  description?: string | CleanTranslation
  error?: string
  class?: HTMLAttributes['class']
  inputClass?: HTMLAttributes['class']

  // Input props
  autoSize?: boolean | number
  modelValue?: string | number
  defaultValue?: string | number
  placeholder?: unknown
  disabled?: boolean
  readonly?: boolean
  rows?: number
  name: string
}>()

const isFullscreen = ref(false)

const emits = defineEmits<{
  (e: 'update:modelValue', payload: string | number): void
}>()

const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
  defaultValue: props.defaultValue,
})

const inputProps = computed(() => {
  const {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    id, label, tooltip, description, error, class: _class,
    modelValue: _modelValue, defaultValue: _defaultValue,
    ...rest
  } = props

  return { ...rest, class: props.inputClass }
})

const handleFullscreenToggle = () => {
  isFullscreen.value = !isFullscreen.value
}

const handleCloseFullscreen = () => {
  isFullscreen.value = false
}

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape' && isFullscreen.value) {
    handleCloseFullscreen()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
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
      <div class="relative">
        <SimpleTooltip
          :tooltip="$t(isFullscreen ? 'common.fullscreenEditor.close' : 'common.fullscreenEditor.open')"
        >
          <Button
            class="absolute top-1 right-1 z-10"
            size="xs"
            tabindex="-1"
            variant="ghost"
            @click="handleFullscreenToggle"
          >
            <Icon
              :name="isFullscreen ? 'lucide:minimize-2' : 'lucide:maximize-2'"
            />
          </Button>
        </SimpleTooltip>
        <Textarea
          :id="id"
          v-model="modelValue"
          :class="{ 'border-destructive': hasError }"
          v-bind="{ ...inputProps, ...$attrs }"
        />
      </div>
      <Dialog
        :open="isFullscreen"
        @update:open="handleCloseFullscreen"
      >
        <DialogContent
          class="max-w-[95vw] h-[90vh] max-h-[90vh] p-3 pt-12 flex flex-col"
        >
          <Textarea
            v-model="modelValue"
            class="flex-1 resize-none"
            v-bind="{ ...inputProps, ...$attrs, rows: undefined }"
          />
        </DialogContent>
      </Dialog>
    </template>
  </FormField>
</template>