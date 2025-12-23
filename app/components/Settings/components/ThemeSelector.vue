<script lang="ts" setup>

import SelectField from '~/components/ui/form/SelectField.vue'

const { themes, currentTheme, previewTheme, switchTheme, isLoading: themeLoading } = useTheme()

const selectedTheme = ref<string>(currentTheme.value)

const handleThemePreview = async (themeId?: string = undefined) => {
  try {
    await previewTheme(themeId ?? selectedTheme.value)
  } catch (_) {
    // Ignore preview errors
  }
}

const handleThemeChange = async (themeId: string) => {
  try {
    await switchTheme(themeId)
  } catch (_) {
    // ignore switch errors
  }
}

const themeOptions = computed(() => {
  return themes.value.map(theme => ({
    value: theme.id,
    label: `${theme.name} (${theme.source})`
  }))
})
</script>

<template>
  <SelectField
    v-model="selectedTheme"
    :disabled="themeLoading"
    :options="themeOptions"
    name="theme"
    placeholder="Choose a theme"
    @update:open="isOpen => isOpen || handleThemePreview()"
    @update:model-value="handleThemeChange"
    @focus-item="({value}) => handleThemePreview(value)"
  />
</template>