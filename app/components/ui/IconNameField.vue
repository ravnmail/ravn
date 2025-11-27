<script setup lang="ts">

import IconGrid from '~/components/ui/IconGrid.vue'
import ColorSelect from '~/components/ui/ColorSelect.vue'
import { Input } from '~/components/ui/input'
import { FormField } from '~/components/ui/form'
import type { CleanTranslation } from 'nuxt-i18n-micro-types/src'

const props = defineProps<{
  modelValue: {
    icon?: string
    color?: string
    name?: string
  }
  label?: string | CleanTranslation
  name?: string | CleanTranslation
}>()

const emit = defineEmits<{
  'update:modelValue': [unknown],
  'update:name': [unknown],
  'update:color': [unknown],
  'update:icon': [unknown]
}>()

const localValue = ref({ ...props.modelValue })

watch(() => props.modelValue, (val) => {
  localValue.value = { ...val }
}, { deep: true })

const update = (key: keyof typeof localValue.value, value: unknown) => {
  localValue.value[key] = value
  emit('update:modelValue', { ...localValue.value })
  emit(`update:${key}`, value)
}
</script>

<template>
  <FormField
    :label="label"
    :name="name"
  >
    <div class="flex gap-2">
      <IconGrid
        :model-value="localValue.icon"
        @update:model-value="update('icon', $event)"
      />
      <ColorSelect
        :model-value="localValue.color"
        @update:model-value="update('color', $event)"
      />
      <Input
        :model-value="localValue.name"
        @update:model-value="update('name', $event)"
      />
    </div>
  </FormField>
</template>

