<script lang="ts" setup>

import { useMousetrap } from '~/lib/moustrap-vue'
import Shortcuts from '~/components/ui/kbd/Shortcuts.vue'
import FormField from '~/components/ui/form/FormField.vue'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import { Button } from '~/components/ui/button'

const { isRecording, record, stopRecording, recordedSequence } = useMousetrap()

const props = defineProps<{
  label: CleanTranslation | string,
  modelValue: string | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | null): void
}>()

const keys = computed(() => {
  if (isRecording.value && recordedSequence.value.length > 0) {
    return recordedSequence.value.join(" ")
  }
  if (props.modelValue) {
    return props.modelValue
  }
  return ''
})

function toggleRecording() {
  if (isRecording.value) {
    stopRecording()
    return
  }

  record((sequence) => {
    emit('update:modelValue', sequence.join(' '))
  })
}

</script>

<template>
  <FormField
    id="keystroke-recorder"
    :label="label"
  >
    <div
      class="flex items-center w-full border border-input px-3 py-2 rounded-md relative"
    >
      <div
        v-if="isRecording"
        :class="[isRecording && 'opacity-50', 'absolute left-1 flex text-2xs text-destructive border border-destructive px-1 rounded-sm gap-1']"
      >
        <span>●</span>
        <span>REC</span>
      </div>
      <div
        class="flex-1 flex items-center justify-center"
      >
        <Shortcuts :keys="keys"/>
      </div>
      <Button
        class="flex absolute right-1 z-10"
        size="bar"
        variant="ghost"
        @click="toggleRecording"
      >
        <Icon
          :class="[isRecording ? 'text-destructive' : 'text-primary']"
          :name="isRecording ? 'lucide:square' : 'lucide:play'"
          :size="14"
        />
      </Button>
    </div>
  </FormField>
</template>