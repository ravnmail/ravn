import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import { toast } from 'vue-sonner'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type { Account, AccountType, CreateAccountRequest, CredentialsRequiredEvent } from '~/types/sync'
import { useSetupQueryListeners } from './useQueryListeners'

const QUERY_KEYS = {
  all: ['accounts'] as const,
  lists: () => [...QUERY_KEYS.all, 'list'] as const,
  list: (filters?: string) => [...QUERY_KEYS.lists(), { filters }] as const,
  details: () => [...QUERY_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...QUERY_KEYS.details(), id] as const,
}

let unlistenCredentials: (() => void) | null = null

export function useAccounts() {
  const { t } = useI18n()
  const router = useRouter()
  const { startOAuth2 } = useAuth()
  const queryClient = useQueryClient()

  const handleCredentialsRequired = async (event: { payload: CredentialsRequiredEvent }) => {
    const { account_id, provider } = event.payload

    try {
      const accounts = await invoke<Account[]>('get_accounts')
      const account = accounts.find(a => a.id === account_id)

      if (!account) {
        console.warn('[useAccounts] Account not found:', account_id)
        return
      }

      const accountName = account.name || account.email
      const providerKey = provider.toLowerCase() as 'gmail' | 'office365' | 'imap' | 'apple'

      const message = t('credentials.errors.missing', { account: accountName }) as string
      const description = t('credentials.errors.missingDescription') as string
      const actionLabel = t(`credentials.providers.${providerKey}.actionLabel`)

      const handleFix = async () => {
        if (provider === 'gmail' || provider === 'office365') {
          try {
            await startOAuth2(provider as AccountType, account_id)
          }
          catch (err) {
            console.error('[useAccounts] OAuth flow failed:', err)
            toast.error(t('common.states.processing') as string)
          }
        }
        else if (provider === 'imap' || provider === 'apple') {
          navigateToAccountSettings(account_id)
        }
      }

      toast(message, {
        description,
        action: {
          label: actionLabel,
          onClick: handleFix,
        },
        duration: Number.POSITIVE_INFINITY,
        id: `credentials-${account_id}`,
      })
    }
    catch (err) {
      console.error('[useAccounts] Failed to handle credentials required:', err)
    }
  }

  const navigateToAccountSettings = (accountId: string) => {
    router.push(`/settings/accounts/${accountId}`)
  }

  const setupCredentialsListener = async () => {
    try {
      unlistenCredentials = await listen('credentials:required', handleCredentialsRequired)
      console.log('[useAccounts] Listening for credentials:required events')
    }
    catch (err) {
      console.error('[useAccounts] Failed to setup credentials listener:', err)
    }
  }

  const cleanupCredentialsListener = () => {
    if (unlistenCredentials) {
      unlistenCredentials()
      console.log('[useAccounts] Cleaned up credentials:required listener')
    }
  }

  useSetupQueryListeners(queryClient, [
    { name: 'account:created', invalidateKey: QUERY_KEYS.lists() },
    { name: 'account:updated', invalidateKey: QUERY_KEYS.lists() },
    { name: 'account:deleted', invalidateKey: QUERY_KEYS.lists() },
  ])

  onMounted(() => {
    setupCredentialsListener()
  })

  onUnmounted(() => {
    cleanupCredentialsListener()
  })

  const {
    data: accounts,
    isLoading,
    error,
    refetch: refetchAccounts,
  } = useQuery({
    queryKey: QUERY_KEYS.lists(),
    queryFn: async () => {
      return await invoke<Account[]>('get_accounts')
    },
  })

  const createAccountMutation = useMutation({
    mutationFn: async (request: CreateAccountRequest) => {
      return await invoke<Account>('create_account', { request })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.lists() })
    },
  })

  const deleteAccountMutation = useMutation({
    mutationFn: async (accountId: string) => {
      return await invoke<string>('delete_account', { accountId })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.lists() })
    },
  })

  return {
    accounts: computed(() => accounts.value || []),
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),
    refetchAccounts,
    createAccount: createAccountMutation.mutateAsync,
    createAccountMutation,
    deleteAccount: deleteAccountMutation.mutateAsync,
    deleteAccountMutation,
    navigateToAccountSettings,
  }
}
