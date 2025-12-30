import { invoke } from '@tauri-apps/api/core'

interface LicenseStatus {
  is_licensed: boolean
  mode: 'opensource' | 'licensed' | 'trial' | 'unlicensed'
}

export default defineNuxtRouteMiddleware(async (to) => {
  if (import.meta.server || to.path.startsWith('/onboarding')) {
    return
  }

  try {
    const status = await invoke<LicenseStatus>('license_status')
    if (status.is_licensed || status.mode === 'opensource') {
      return
    }

    return navigateTo('/onboarding')
  } catch (error) {
    return navigateTo('/onboarding')
  }
})
