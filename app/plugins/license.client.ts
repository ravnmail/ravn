import { invoke } from '@tauri-apps/api/core'

export default defineNuxtPlugin(async () => {
  if (import.meta.server) {
    return
  }

  try {
    const status = await invoke('license_status')
    console.log('License status initialized:', status)
  } catch (error) {
    console.error('Failed to initialize license status:', error)
  }
})