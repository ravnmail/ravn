<script lang="ts" setup>
import dayjs from 'dayjs'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'

import { Button } from '~/components/ui/button'
import { FormField } from '~/components/ui/form'
import { Input } from '~/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select'
import type {
  ReminderPresetSetting,
  ReminderPresetType,
  ReminderPresetUnit,
} from '~/types/settings'

interface Props {
  modelValue?: ReminderPresetSetting[]
  name?: string
  label?: string | CleanTranslation
  description?: string | CleanTranslation
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: () => [],
  name: 'email.reminders.presets',
  label: 'settings.email.reminders.presets.name',
  description: 'settings.email.reminders.presets.description',
  disabled: false,
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: ReminderPresetSetting[]): void
}>()

const presetTypeOptions: Array<{ value: ReminderPresetType; label: string }> = [
  { value: 'laterToday', label: 'Later today' },
  { value: 'tomorrow', label: 'Tomorrow' },
  { value: 'nextWeek', label: 'Next week' },
  { value: 'nextMonth', label: 'Next month' },
  { value: 'custom', label: 'Custom' },
]

const unitOptions: Array<{ value: ReminderPresetUnit; label: string }> = [
  { value: 'minute', label: 'Minutes' },
  { value: 'hour', label: 'Hours' },
  { value: 'day', label: 'Days' },
  { value: 'week', label: 'Weeks' },
  { value: 'month', label: 'Months' },
]

const defaultIconByType: Record<ReminderPresetType, string> = {
  laterToday: 'lucide:clock-3',
  tomorrow: 'lucide:sunrise',
  nextWeek: 'lucide:calendar-days',
  nextMonth: 'lucide:calendar-range',
  custom: 'lucide:bell',
}

const customDefaultPreset = (): ReminderPresetSetting => ({
  id: `preset-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
  type: 'custom',
  icon: defaultIconByType.custom,
  offset: {
    value: 1,
    unit: 'day',
  },
  time: '09:00',
})

const normalizePreset = (
  preset: Partial<ReminderPresetSetting> | undefined,
  index: number
): ReminderPresetSetting => {
  const type = (preset?.type ?? 'custom') as ReminderPresetType

  return {
    id: preset?.id?.trim() || `preset-${index + 1}`,
    type,
    icon: preset?.icon?.trim() || defaultIconByType[type],
    offset:
      type === 'custom'
        ? {
            value: Math.max(1, Number(preset?.offset?.value) || 1),
            unit: (preset?.offset?.unit ?? 'day') as ReminderPresetUnit,
          }
        : undefined,
    time:
      type === 'tomorrow' || type === 'nextWeek' || type === 'nextMonth' || type === 'custom'
        ? preset?.time?.trim() || '09:00'
        : undefined,
  }
}

const items = computed<ReminderPresetSetting[]>({
  get: () => (props.modelValue ?? []).map((preset, index) => normalizePreset(preset, index)),
  set: (value) => emit('update:modelValue', value),
})

const isCustomType = (type: ReminderPresetType) => type === 'custom'

const usesTime = (type: ReminderPresetType) =>
  type === 'tomorrow' || type === 'nextWeek' || type === 'nextMonth' || type === 'custom'

const updatePreset = (index: number, patch: Partial<ReminderPresetSetting>) => {
  if (props.disabled) return

  items.value = items.value.map((item, currentIndex) => {
    if (currentIndex !== index) return item

    const nextType = (patch.type ?? item.type) as ReminderPresetType
    const nextPreset: ReminderPresetSetting = {
      ...item,
      ...patch,
      type: nextType,
      icon:
        patch.icon !== undefined
          ? patch.icon
          : item.icon && item.icon !== defaultIconByType[item.type]
            ? item.icon
            : defaultIconByType[nextType],
    }

    if (!isCustomType(nextType)) {
      nextPreset.offset = undefined
    }
    if (!usesTime(nextType)) {
      nextPreset.time = undefined
    } else if (!nextPreset.time) {
      nextPreset.time = '09:00'
    }

    if (isCustomType(nextType) && !nextPreset.offset) {
      nextPreset.offset = { value: 1, unit: 'day' }
    }

    return nextPreset
  })
}

const updateOffset = (
  index: number,
  patch: Partial<NonNullable<ReminderPresetSetting['offset']>>
) => {
  if (props.disabled) return

  items.value = items.value.map((item, currentIndex) => {
    if (currentIndex !== index) return item

    const currentOffset = item.offset ?? { value: 1, unit: 'day' as ReminderPresetUnit }

    return {
      ...item,
      offset: {
        ...currentOffset,
        ...patch,
      },
    }
  })
}

const addPreset = () => {
  if (props.disabled) return
  items.value = [...items.value, customDefaultPreset()]
}

const removePreset = (index: number) => {
  if (props.disabled) return
  items.value = items.value.filter((_, currentIndex) => currentIndex !== index)
}

const movePreset = (index: number, direction: -1 | 1) => {
  if (props.disabled) return

  const targetIndex = index + direction
  if (targetIndex < 0 || targetIndex >= items.value.length) return

  const next = [...items.value]
  const [item] = next.splice(index, 1)
  if (!item) return
  next.splice(targetIndex, 0, item)
  items.value = next
}

const formatTimeLabel = (time?: string) => {
  if (!time) return ''
  const parsed = dayjs(`2000-01-01T${time}`)
  return parsed.isValid() ? parsed.format('HH:mm') : time
}

const previewLabel = (preset: ReminderPresetSetting) => {
  const now = dayjs()

  switch (preset.type) {
    case 'laterToday': {
      return now.add(3, 'hour').minute(0).second(0).format('ddd, MMM D [at] HH:mm')
    }
    case 'tomorrow': {
      const base = now.add(1, 'day')
      const [hour, minute] = (preset.time ?? '09:00').split(':').map((v) => Number(v) || 0)
      return base.hour(hour).minute(minute).second(0).format('ddd, MMM D [at] HH:mm')
    }
    case 'nextWeek': {
      const base = now.add(1, 'week')
      const [hour, minute] = (preset.time ?? '09:00').split(':').map((v) => Number(v) || 0)
      return base.hour(hour).minute(minute).second(0).format('ddd, MMM D [at] HH:mm')
    }
    case 'nextMonth': {
      const base = now.add(1, 'month').date(1)
      const [hour, minute] = (preset.time ?? '09:00').split(':').map((v) => Number(v) || 0)
      return base.hour(hour).minute(minute).second(0).format('ddd, MMM D [at] HH:mm')
    }
    case 'custom': {
      const value = Math.max(1, Number(preset.offset?.value) || 1)
      const unit = (preset.offset?.unit ?? 'day') as ReminderPresetUnit
      const [hour, minute] = (preset.time ?? '09:00').split(':').map((v) => Number(v) || 0)
      return now
        .add(value, unit)
        .hour(hour)
        .minute(minute)
        .second(0)
        .format('ddd, MMM D [at] HH:mm')
    }
  }
}

const helperText = (preset: ReminderPresetSetting) => {
  if (preset.type === 'custom') {
    const value = Math.max(1, Number(preset.offset?.value) || 1)
    const unit = preset.offset?.unit ?? 'day'
    return `In ${value} ${unit}${value === 1 ? '' : 's'} · ${formatTimeLabel(preset.time)}`
  }

  if (usesTime(preset.type)) {
    return formatTimeLabel(preset.time)
  }

  return preset.type
}
</script>

<template>
  <FormField
    :description="description"
    :label="label"
    :name="name"
  >
    <div class="space-y-3">
      <div
        v-if="items.length"
        class="space-y-3"
      >
        <div
          v-for="(preset, index) in items"
          :key="preset.id"
          class="rounded-lg border bg-card p-4"
        >
          <div class="flex items-start gap-3">
            <div class="min-w-0 flex-1 space-y-3">
              <div class="grid grid-cols-1 gap-3 md:grid-cols-[180px_1fr_160px]">
                <div class="space-y-1">
                  <div class="text-sm font-medium">Type</div>
                  <Select
                    :disabled="disabled"
                    :model-value="preset.type"
                    @update:model-value="
                      updatePreset(index, { type: $event as ReminderPresetType })
                    "
                  >
                    <SelectTrigger class="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="option in presetTypeOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="space-y-1">
                  <div class="text-sm font-medium">ID</div>
                  <Input
                    :disabled="disabled"
                    :model-value="preset.id"
                    placeholder="later-today"
                    @update:model-value="
                      updatePreset(index, { id: String($event).trim() || preset.id })
                    "
                  />
                </div>

                <div class="space-y-1">
                  <div class="text-sm font-medium">Icon</div>
                  <Input
                    :disabled="disabled"
                    :model-value="preset.icon ?? ''"
                    placeholder="lucide:clock-3"
                    @update:model-value="updatePreset(index, { icon: String($event).trim() })"
                  />
                </div>
              </div>

              <div
                v-if="isCustomType(preset.type)"
                class="grid grid-cols-1 gap-3 md:grid-cols-[140px_160px_160px]"
              >
                <div class="space-y-1">
                  <div class="text-sm font-medium">Offset</div>
                  <Input
                    :disabled="disabled"
                    :min="1"
                    :model-value="String(preset.offset?.value ?? 1)"
                    type="number"
                    @update:model-value="
                      updateOffset(index, { value: Math.max(1, Number($event) || 1) })
                    "
                  />
                </div>

                <div class="space-y-1">
                  <div class="text-sm font-medium">Unit</div>
                  <Select
                    :disabled="disabled"
                    :model-value="preset.offset?.unit ?? 'day'"
                    @update:model-value="
                      updateOffset(index, { unit: $event as ReminderPresetUnit })
                    "
                  >
                    <SelectTrigger class="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="option in unitOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="space-y-1">
                  <div class="text-sm font-medium">Time</div>
                  <Input
                    :disabled="disabled"
                    :model-value="preset.time ?? '09:00'"
                    type="time"
                    @update:model-value="updatePreset(index, { time: String($event) })"
                  />
                </div>
              </div>

              <div
                v-else-if="usesTime(preset.type)"
                class="grid grid-cols-1 gap-3 md:grid-cols-[160px_1fr]"
              >
                <div class="space-y-1">
                  <div class="text-sm font-medium">Time</div>
                  <Input
                    :disabled="disabled"
                    :model-value="preset.time ?? '09:00'"
                    type="time"
                    @update:model-value="updatePreset(index, { time: String($event) })"
                  />
                </div>

                <div class="text-muted-foreground flex min-h-10 items-end text-sm">
                  {{ helperText(preset) }}
                </div>
              </div>

              <div class="rounded-md bg-muted/50 px-3 py-2">
                <div class="text-sm font-medium">Preview</div>
                <div class="text-muted-foreground text-sm">
                  {{ previewLabel(preset) }}
                </div>
              </div>
            </div>

            <div class="flex shrink-0 items-center gap-1">
              <Button
                :disabled="disabled || index === 0"
                size="icon"
                type="button"
                variant="ghost"
                @click="movePreset(index, -1)"
              >
                <Icon name="lucide:arrow-up" />
              </Button>

              <Button
                :disabled="disabled || index === items.length - 1"
                size="icon"
                type="button"
                variant="ghost"
                @click="movePreset(index, 1)"
              >
                <Icon name="lucide:arrow-down" />
              </Button>

              <Button
                :disabled="disabled"
                size="icon"
                type="button"
                variant="ghost"
                @click="removePreset(index)"
              >
                <Icon name="lucide:trash-2" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <div
        v-else
        class="text-muted-foreground rounded-lg border border-dashed p-4 text-sm"
      >
        No reminder presets configured yet.
      </div>

      <div class="flex items-center gap-2">
        <Button
          :disabled="disabled"
          type="button"
          variant="outline"
          @click="addPreset"
        >
          <Icon
            class="mr-2"
            name="lucide:plus"
          />
          Add preset
        </Button>
      </div>
    </div>
  </FormField>
</template>
