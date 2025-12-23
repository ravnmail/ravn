import { invoke } from '@tauri-apps/api/core'
import type { Settings, PartialDeep } from '~/types/settings'


export function useSettings() {
  // Global state for settings
  const settings = useState<Settings | null>('settings', () => null)
  const userKeys = useState<Set<string>>('user-setting-keys', () => new Set())
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

  const getUserKeys = async (): Promise<Set<string>> => {
    try {
      const result = await invoke<string[]>('get_user_keys')
      const keysSet = new Set<string>(result)
      userKeys.value = keysSet
      return keysSet
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
      await getUserKeys()
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    }
  }

  const removeSetting = async (key: string) => {
    error.value = null
    try {
      await invoke('remove_setting', { key })
      // Reload all settings to keep state in sync
      await fetchSettings()
      await getUserKeys()
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
      await fetchSettings()
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    }
  }

  const reloadSettings = async () => {
    error.value = null
    try {
      await invoke('reload_settings')
      await fetchSettings()
      await getUserKeys()
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    }
  }

  if (settings.value === null && !isLoading.value) {
    fetchSettings()
    getUserKeys()
  }

  return {
    settings: readonly(settings),
    isLoading: readonly(isLoading),
    error: readonly(error),
    userKeys: readonly(userKeys),

    isUserSetting: (key: string) => userKeys.value.has(key),

    fetchSettings,
    getSetting,
    setSetting,
    setSettings,
    removeSetting,
    reloadSettings,
  }
}