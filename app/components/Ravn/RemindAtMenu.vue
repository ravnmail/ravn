<script lang="ts" setup>
import dayjs from 'dayjs'

import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '~/components/ui/command'
import type { ReminderPresetSetting } from '~/types/settings'

interface ReminderMenuItem {
  id: string
  label: string
  icon: string
  remindAt: string | null
}

const props = withDefaults(
  defineProps<{
    modelValue?: string | null
    presets?: ReminderPresetSetting[] | null
    includeClear?: boolean
    includeCustom?: boolean
  }>(),
  {
    modelValue: null,
    presets: null,
    includeClear: true,
    includeCustom: true,
  }
)

const emit = defineEmits<{
  (e: 'select', value: string | null): void
  (e: 'update:modelValue', value: string | null): void
  (e: 'close'): void
}>()

const { t } = useI18n()
const { settings } = useSettings()

const customValue = ref('')

const activePresets = computed<ReminderPresetSetting[]>(() => {
  return props.presets ?? settings.value?.email?.reminders?.presets ?? []
})

watch(
  () => props.modelValue,
  (value) => {
    customValue.value = value ? toDatetimeLocalValue(value) : ''
  },
  { immediate: true }
)

const toDatetimeLocalValue = (value: string) => {
  const parsed = dayjs(value)
  if (!parsed.isValid()) return ''
  return parsed.format('YYYY-MM-DDTHH:mm')
}

const parseTime = (value?: string | null) => {
  const match = value?.match(/^(\d{1,2}):(\d{2})$/)
  if (!match) return null

  const hours = Number(match[1])
  const minutes = Number(match[2])

  if (
    Number.isNaN(hours) ||
    Number.isNaN(minutes) ||
    hours < 0 ||
    hours > 23 ||
    minutes < 0 ||
    minutes > 59
  ) {
    return null
  }

  return { hours, minutes }
}

const applyTime = (base: dayjs.Dayjs, time?: string | null) => {
  const parsedTime = parseTime(time)
  if (!parsedTime) return base

  return base.hour(parsedTime.hours).minute(parsedTime.minutes).second(0).millisecond(0)
}

const resolvePresetDate = (preset: ReminderPresetSetting): string | null => {
  const now = dayjs()

  switch (preset.type) {
    case 'laterToday': {
      const hours = Math.max(1, preset.offset?.value ?? 3)
      return now.add(hours, 'hour').minute(0).second(0).millisecond(0).toISOString()
    }

    case 'tomorrow': {
      return applyTime(now.add(1, 'day').startOf('day'), preset.time ?? '09:00').toISOString()
    }

    case 'nextWeek': {
      return applyTime(now.add(1, 'week').startOf('day'), preset.time ?? '09:00').toISOString()
    }

    case 'nextMonth': {
      return applyTime(
        now.add(1, 'month').date(1).startOf('day'),
        preset.time ?? '09:00'
      ).toISOString()
    }

    case 'custom': {
      const unit = preset.offset?.unit ?? 'day'
      const value = Math.max(1, preset.offset?.value ?? 1)
      return applyTime(now.add(value, unit), preset.time).second(0).millisecond(0).toISOString()
    }

    default:
      return null
  }
}

const formatAbsoluteLabel = (value: string) => {
  const parsed = dayjs(value)
  if (!parsed.isValid()) return value
  return parsed.format('ddd, MMM D • HH:mm')
}

const getPresetLabel = (preset: ReminderPresetSetting, remindAt: string | null) => {
  if (preset.label?.trim()) return preset.label.trim()

  switch (preset.type) {
    case 'laterToday':
      return t('components.remindAt.laterToday')
    case 'tomorrow':
      return t('components.remindAt.tomorrow')
    case 'nextWeek':
      return t('components.remindAt.nextWeek')
    case 'nextMonth':
      return t('components.remindAt.nextMonth')
    case 'custom':
      return remindAt ? formatAbsoluteLabel(remindAt) : t('components.remindAt.custom')
    default:
      return remindAt ? formatAbsoluteLabel(remindAt) : ''
  }
}

const reminderItems = computed<ReminderMenuItem[]>(() => {
  const items = activePresets.value
    .map((preset, index) => {
      const remindAt = resolvePresetDate(preset)
      if (!remindAt) return null

      return {
        id: preset.id || `${preset.type}-${index}`,
        icon: preset.icon?.trim() || 'lucide:clock-3',
        label: getPresetLabel(preset, remindAt),
        remindAt,
      }
    })
    .filter((item): item is ReminderMenuItem => item !== null)

  if (props.includeClear && !items.some((item) => item.remindAt === null)) {
    items.push({
      id: 'clear',
      icon: 'lucide:x-circle',
      label: t('components.remindAt.clear'),
      remindAt: null,
    })
  }

  return items
})

const selectedPresetId = computed(() => {
  if (!props.modelValue) {
    return reminderItems.value.find((item) => item.remindAt === null)?.id ?? null
  }

  return reminderItems.value.find((item) => item.remindAt === props.modelValue)?.id ?? null
})

const applySelection = (value: string | null) => {
  emit('update:modelValue', value)
  emit('select', value)
  emit('close')
}

const handlePresetSelect = (value: string | null) => {
  applySelection(value)
}

const handleCustomApply = () => {
  if (!customValue.value) return

  const parsed = dayjs(customValue.value)
  if (!parsed.isValid()) return

  applySelection(parsed.second(0).millisecond(0).toISOString())
}
</script>

<template>
  <Command class="w-80">
    <CommandInput
      class="py-2 text-sm"
      :placeholder="t('components.remindAt.searchPlaceholder')"
    />

    <CommandList>
      <CommandEmpty>{{ t('components.remindAt.empty') }}</CommandEmpty>

      <CommandGroup>
        <CommandItem
          v-for="item in reminderItems"
          :key="item.id"
          :value="`${item.label} ${item.remindAt ?? ''}`"
          class="flex items-center gap-2"
          @select.prevent="handlePresetSelect(item.remindAt)"
        >
          <Icon
            :name="item.icon"
            class="shrink-0"
            size="16"
          />

          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium">
              {{ item.label }}
            </div>
            <div
              v-if="item.remindAt"
              class="text-muted-foreground truncate text-xs"
            >
              {{ formatAbsoluteLabel(item.remindAt) }}
            </div>
          </div>

          <Icon
            v-if="selectedPresetId === item.id"
            class="ml-auto shrink-0"
            name="lucide:check"
            size="16"
          />
        </CommandItem>
      </CommandGroup>

      <div
        v-if="includeCustom"
        class="border-t p-3"
      >
        <div class="mb-2 flex items-center gap-2 text-sm font-medium">
          <Icon
            name="lucide:calendar-clock"
            size="16"
          />
          <span>{{ t('components.remindAt.custom') }}</span>
        </div>

        <div class="space-y-2">
          <input
            v-model="customValue"
            class="placeholder:text-muted-foreground flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors outline-none file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:cursor-not-allowed disabled:opacity-50"
            type="datetime-local"
            @keydown.enter.prevent="handleCustomApply"
          />

          <button
            class="relative flex w-full cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none select-none hover:bg-selection hover:text-selection-foreground focus-visible:bg-selection focus-visible:text-selection-foreground"
            type="button"
            @click.stop="handleCustomApply"
          >
            <Icon
              class="mr-2"
              name="lucide:check"
              size="16"
            />
            <span>{{ t('components.remindAt.custom') }}</span>
          </button>
        </div>
      </div>
    </CommandList>
  </Command>
</template>
