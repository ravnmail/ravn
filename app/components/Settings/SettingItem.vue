<script lang="ts" setup>
import type { SettingGroup, SettingItem as SettingItemType } from '~/types/settings-manifest'
import { resolveSettingComponent } from '~/utils/settings/component-mapper'
import SettingItemHeader from '~/components/Settings/SettingItemHeader.vue'

const props = defineProps<{
  item: SettingItemType
  group: SettingGroup
}>()

const { t } = useI18n()
const { settings, removeSetting, setSetting } = useSettings()

const currentValue = computed(() => {
  if (!settings.value) return undefined

  const ids = props.item.id.split('.')
  let value: never = settings.value

  for (const id of ids) {
    value = value?.[id]
  }

  return value
})

const localValue = ref(currentValue.value)

watch(currentValue, (newValue) => {
  localValue.value = newValue
}, { immediate: true })

let saveTimeout: ReturnType<typeof setTimeout> | null = null

function handleChange(newValue: any) {
  localValue.value = newValue
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }

  saveTimeout = setTimeout(async () => {
    try {
      await setSetting(props.item.id, newValue)
    } catch (err) {
      console.error(`Failed to save setting ${props.item.id}:`, err)
    }
  }, 500)
}

async function handleReset() {
  try {
    await removeSetting(props.item.id)
  } catch (err) {
    console.error(`Failed to reset setting ${props.item.id}:`, err)
  }
}

onUnmounted(() => {
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }
})

const settingComponent = computed(() => resolveSettingComponent(props.item.is))

</script>

<template>
  <div
    :id="item.id"
    class="flex items-start"
  >
    <SettingItemHeader
      :description="item.description"
      :group="group"
      :setting-key="item.id"
      :title="item.name"
      class="-ml-7"
      @reset="handleReset"
    />

    <div class="ml-auto max-w-1/2">
      <component
        :is="settingComponent"
        v-model="localValue"
        :disabled="item.disabled"
        v-bind="item.props"
        @update:model-value="handleChange"
      />
    </div>
  </div>
</template>
