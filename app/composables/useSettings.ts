import { invoke } from '@tauri-apps/api/core'
import type { Settings, PartialDeep } from '~/types/settings'


export function useSettings() {
  // Global state for settings
  const settings = useState<Settings | null>('settings', () => null)
  const isLoading = useState<boolean>('settings-loading', () => false)
  const error = useState<string | null>('settings-error', () => null)

  // Fetch all settings from backend
  const fetchSettings = async () => {
    isLoading.value = true
    error.value = null
    try {
      const result = await invoke<Settings>('get_all_settings')
      settings.value = result
      return result
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    }
    finally {
      isLoading.value = false
    }
  }

  // Get a specific setting by key (dot notation)
  const getSetting = async <T = any>(key: string): Promise<T> => {
    try {
      const result = await invoke<T>('get_setting', { key })
      return result
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    }
  }

  // Set a specific setting by key (dot notation)
  const setSetting = async (key: string, value: any) => {
    error.value = null
    try {
      await invoke('set_setting', { key, value })
      // Reload all settings to keep state in sync
      await fetchSettings()
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    }
  }

  // Set multiple settings at once
  const setSettings = async (newSettings: PartialDeep<Settings>) => {
    error.value = null
    try {
      await invoke('set_settings', { settings: newSettings })
      // Reload all settings to keep state in sync
      await fetchSettings()
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    }
  }

  // Reload settings from disk (useful if file was changed externally)
  const reloadSettings = async () => {
    error.value = null
    try {
      await invoke('reload_settings')
      await fetchSettings()
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    }
  }

  // Auto-fetch settings on first use if not already loaded
  if (settings.value === null && !isLoading.value) {
    fetchSettings()
  }

  return {
    // State
    settings: readonly(settings),
    isLoading: readonly(isLoading),
    error: readonly(error),

    // Methods
    fetchSettings,
    getSetting,
    setSetting,
    setSettings,
    reloadSettings,
  }
}

// Composable for working with a specific setting category
export function useCategorySettings<K extends keyof Settings>(category: K) {
  const { settings, isLoading, error, setSetting, fetchSettings } = useSettings()

  const categorySettings = computed(() => {
    return settings.value?.[category] ?? null
  })

  const updateCategorySetting = async (key: string, value: any) => {
    await setSetting(`${category}.${key}`, value)
  }

  const updateCategory = async (newCategorySettings: Partial<Settings[K]>) => {
    const updates: Record<string, any> = {}
    for (const [key, value] of Object.entries(newCategorySettings)) {
      updates[`${category}.${key}`] = value
    }

    for (const [key, value] of Object.entries(updates)) {
      await setSetting(key, value)
    }
  }

  return {
    settings: categorySettings,
    isLoading,
    error,
    updateSetting: updateCategorySetting,
    updateCategory,
    refetch: fetchSettings,
  }
}
