import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import type {
  AccountType,
  ImapConnectionConfig,
  StartOAuth2Request,
  StartOAuth2Response,
  StoreImapCredentialsRequest,
  SyncReport,
} from '~/types/sync'

export function useAuth() {
  const isAuthenticating = ref(false)
  const error = ref<string | null>(null)
  const oauthState = ref<{
    csrf_token?: string
    provider?: string
    account_id?: string
  }>({})

  const startOAuth2 = async (
    provider: AccountType,
    accountId: string,
    redirectUri: string = 'https://coderscantina-ravn.s3.eu-west-1.amazonaws.com/oauth-callback.html'
  ) => {
    isAuthenticating.value = true
    error.value = null

    try {
      const request: StartOAuth2Request = {
        provider,
        redirect_uri: redirectUri,
      }

      const response = await invoke<StartOAuth2Response>('start_oauth2_flow', {
        request,
        accountId,
      })

      if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
        localStorage.setItem('oauth_csrf_token', response.csrf_token)
        localStorage.setItem('oauth_provider', provider)
        localStorage.setItem('oauth_account_id', accountId)
      }

      await invoke('open_oauth_window', {
        authUrl: response.auth_url,
        provider,
      })

      return response
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw new Error(errorMessage)
    }
    finally {
      isAuthenticating.value = false
    }
  }

  const exchangeOAuth2Code = async (
    code: string,
  ) => {
    isAuthenticating.value = true
    error.value = null

    try {
      const csrfToken = typeof localStorage !== 'undefined'
        ? localStorage.getItem('oauth_csrf_token')
        : null

      if (!csrfToken) {
        throw new Error('OAuth state not found. Please restart the authentication flow.')
      }

      const response = await invoke<string>('exchange_oauth2_code', {
        code,
        csrfToken,
      })

      if (typeof localStorage !== 'undefined') {
        localStorage.removeItem('oauth_csrf_token')
        localStorage.removeItem('oauth_provider')
        localStorage.removeItem('oauth_account_id')
      }

      return response
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw new Error(errorMessage)
    }
    finally {
      isAuthenticating.value = false
    }
  }

  /**
   * Store IMAP credentials
   */
  const storeImapCredentials = async (
    accountId: string,
    config: ImapConnectionConfig
  ) => {
    isAuthenticating.value = true
    error.value = null

    try {
      const request: StoreImapCredentialsRequest = {
        account_id: accountId,
        username: config.username,
        password: config.password,
      }

      const response = await invoke<string>('store_imap_credentials', {
        request,
      })

      return response
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw new Error(errorMessage)
    }
    finally {
      isAuthenticating.value = false
    }
  }

  return {
    isAuthenticating: readonly(isAuthenticating),
    error: readonly(error),
    oauthState: readonly(oauthState),
    startOAuth2,
    exchangeOAuth2Code,
    storeImapCredentials,
  }
}

export function useSync() {
  const isSyncing = ref(false)
  const error = ref<string | null>(null)

  const syncAccount = async (accountId: string) => {
    isSyncing.value = true
    error.value = null

    try {
      const report = await invoke<SyncReport>('sync_account', {
        accountId,
      })

      return report
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw new Error(errorMessage)
    }
    finally {
      isSyncing.value = false
    }
  }

  const syncFolder = async (
    accountId: string,
    folderId: string,
    full?: boolean
  ) => {
    isSyncing.value = true
    error.value = null

    try {
      const count = await invoke<number>('sync_folder', {
        accountId,
        folderId,
        full,
      })

      return count
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw new Error(errorMessage)
    }
    finally {
      isSyncing.value = false
    }
  }

  return {
    isSyncing: readonly(isSyncing),
    error: readonly(error),
    syncAccount,
    syncFolder
  }
}

export function useOffice365AuthRefresh() {
  const { startOAuth2 } = useAuth()
  const reAuthRequired = ref(false)
  const reAuthAccountId = ref<string | null>(null)

  let unlistenFn: (() => void) | null = null

  const handleAuthRequired = async (event: any) => {
    const payload = event.payload
    console.log('[Office365] Re-authentication required:', payload)

    reAuthRequired.value = true
    reAuthAccountId.value = payload.account_id

    if (typeof window !== 'undefined' && 'Notification' in window && Notification.permission === 'granted') {
      new Notification('Office365 Authentication Required', {
        body: 'Your Office365 session has expired. Please re-authenticate.',
        icon: '/favicon.ico',
      })
    }

    try {
      await startOAuth2('office365', payload.account_id, 'http://localhost:3000/auth/callback')
      console.log('[Office365] Re-authentication flow initiated')
    }
    catch (err) {
      console.error('[Office365] Failed to initiate re-authentication:', err)
    }
  }

  onMounted(async () => {
    try {
      unlistenFn = await listen('office365:auth-required', handleAuthRequired)
      console.log('[Office365] Auth refresh listener registered')
    }
    catch (err) {
      console.error('[Office365] Failed to register auth refresh listener:', err)
    }
  })

  onUnmounted(() => {
    if (unlistenFn) {
      unlistenFn()
      console.log('[Office365] Auth refresh listener unregistered')
    }
  })

  return {
    reAuthRequired: readonly(reAuthRequired),
    reAuthAccountId: readonly(reAuthAccountId),
  }
}
