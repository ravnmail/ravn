import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface LicenseStatus {
  is_licensed: boolean
  mode: 'opensource' | 'licensed' | 'trial' | 'unlicensed'
  status?: 'active' | 'trial' | 'expired' | 'suspended'
  user_name?: string
  user_email?: string
  ai_mode?: 'saas' | 'byok'
  ai_limit?: number
  ai_limit_remaining?: number
  expires_at?: string
  trial_ends_at?: string
  validated_at?: string
}

export interface LicenseDetails {
  is_licensed: boolean
  mode: string
  key?: string
  status?: string
  user_name?: string
  user_email?: string
  ai_mode?: string
  expires_at?: string
  trial_ends_at?: string
  expiration_date?: string
  is_expired: boolean
  is_trial: boolean
  days_remaining?: number
}

export interface LicenseResponse {
  success: boolean
  message: string
  status?: LicenseStatus
}

export interface ActivateRequest {
  license_key: string
}

export interface TrialRequest {
  email: string
}

export function useLicense() {
  const licenseStatus = useState<LicenseStatus | null>('license-status', () => null)
  const licenseDetails = useState<LicenseDetails | null>('license-details', () => null)
  const isLoading = useState<boolean>('license-loading', () => false)
  const error = useState<string | null>('license-error', () => null)

  const fetchStatus = async () => {
    isLoading.value = true
    error.value = null

    try {
      const status = await invoke<LicenseStatus>('license_status')
      licenseStatus.value = status
      return status
    } catch (e) {
      error.value = e as string
      console.error('Failed to fetch license status:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  const fetchDetails = async () => {
    isLoading.value = true
    error.value = null

    try {
      const details = await invoke<LicenseDetails>('license_details')
      licenseDetails.value = details
      return details
    } catch (e) {
      error.value = e as string
      console.error('Failed to fetch license details:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  const activate = async (licenseKey: string) => {
    isLoading.value = true
    error.value = null

    try {
      const response = await invoke<LicenseResponse>('license_activate', {
        request: { license_key: licenseKey } as ActivateRequest
      })

      if (response.success && response.status) {
        licenseStatus.value = response.status
        await fetchDetails()
      } else {
        error.value = response.message
      }

      return response
    } catch (e) {
      const errorMessage = e as string
      error.value = errorMessage
      console.error('Failed to activate license:', e)
      return {
        success: false,
        message: errorMessage,
        status: undefined
      } as LicenseResponse
    } finally {
      isLoading.value = false
    }
  }

  const startTrial = async (email: string) => {
    isLoading.value = true
    error.value = null

    try {
      const response = await invoke<LicenseResponse>('license_trial', {
        request: { email } as TrialRequest
      })

      if (response.success && response.status) {
        licenseStatus.value = response.status
        await fetchDetails()
      } else {
        error.value = response.message
      }

      return response
    } catch (e) {
      const errorMessage = e as string
      error.value = errorMessage
      console.error('Failed to start trial:', e)
      return {
        success: false,
        message: errorMessage,
        status: undefined
      } as LicenseResponse
    } finally {
      isLoading.value = false
    }
  }

  const validate = async () => {
    isLoading.value = true
    error.value = null

    try {
      const response = await invoke<LicenseResponse>('license_validate')

      if (response.success && response.status) {
        licenseStatus.value = response.status
        await fetchDetails()
      } else {
        error.value = response.message
      }

      return response
    } catch (e) {
      const errorMessage = e as string
      error.value = errorMessage
      console.error('Failed to validate license:', e)
      return {
        success: false,
        message: errorMessage,
        status: undefined
      } as LicenseResponse
    } finally {
      isLoading.value = false
    }
  }

  const clearLicense = async () => {
    isLoading.value = true
    error.value = null

    try {
      const response = await invoke<LicenseResponse>('license_clear')

      if (response.success) {
        licenseStatus.value = null
        licenseDetails.value = null
      } else {
        error.value = response.message
      }

      return response
    } catch (e) {
      const errorMessage = e as string
      error.value = errorMessage
      console.error('Failed to clear license:', e)
      return {
        success: false,
        message: errorMessage,
        status: undefined
      } as LicenseResponse
    } finally {
      isLoading.value = false
    }
  }

  const setupLicenseListener = () => {
    if (import.meta.client) {
      listen('license-updated', async () => {
        console.log('License updated, refreshing status...')
        await fetchStatus()
        await fetchDetails()
      })
    }
  }

  const isLicensed = computed(() => licenseStatus.value?.is_licensed ?? false)
  const isOpenSourceMode = computed(() => licenseStatus.value?.mode === 'opensource')
  const isTrialMode = computed(() => licenseStatus.value?.mode === 'trial')
  const isExpired = computed(() => licenseDetails.value?.is_expired ?? false)
  const aiMode = computed(() => licenseStatus.value?.ai_mode)
  const licenseMode = computed(() => licenseStatus.value?.mode)
  const licenseStatusType = computed(() => licenseStatus.value?.status)

  const expirationDate = computed(() => {
    if (licenseDetails.value?.is_trial) {
      return licenseDetails.value?.trial_ends_at
    }
    return licenseDetails.value?.expires_at
  })

  const daysRemaining = computed(() => licenseDetails.value?.days_remaining)
  const aiLimits = computed(() => {
    if (!licenseStatus.value) return null
    
    return {
      limit: licenseStatus.value.ai_limit ?? 0,
      remaining: licenseStatus.value.ai_limit_remaining ?? 0,
      usage: licenseStatus.value.ai_limit 
        ? ((licenseStatus.value.ai_limit - (licenseStatus.value.ai_limit_remaining ?? 0)) / licenseStatus.value.ai_limit) * 100
        : 0
    }
  })

  const formattedExpirationDate = computed(() => {
    const expDate = expirationDate.value
    if (!expDate) return null

    try {
      const date = new Date(expDate)
      return date.toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'long',
        day: 'numeric'
      })
    } catch (e) {
      console.error('Failed to format expiration date:', e)
      return null
    }
  })

  const expirationStatus = computed(() => {
    const days = daysRemaining.value
    if (days === null || days === undefined) return null

    if (days < 0) return 'expired'
    if (days === 0) return 'expires_today'
    if (days === 1) return 'expires_tomorrow'
    if (days <= 7) return 'expires_soon'
    if (days <= 30) return 'expires_this_month'
    return 'active'
  })

  const needsAttention = computed(() => {
    const status = expirationStatus.value
    return status === 'expired' || status === 'expires_today' || status === 'expires_tomorrow' || status === 'expires_soon'
  })

  onMounted(async () => {
    await fetchStatus()
    await fetchDetails()
    setupLicenseListener()
  })

  return {
    // State
    licenseStatus,
    licenseDetails,
    isLoading,
    error,
    
    // Computed
    isLicensed,
    isOpenSourceMode,
    isTrialMode,
    isExpired,
    aiMode,
    licenseMode,
    licenseStatusType,
    aiLimits,
    expirationDate,
    formattedExpirationDate,
    daysRemaining,
    expirationStatus,
    needsAttention,
    
    // Methods
    fetchStatus,
    fetchDetails,
    activate,
    startTrial,
    validate,
    clearLicense
  }
}