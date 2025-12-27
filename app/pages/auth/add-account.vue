<script lang="ts" setup>
import { ref, computed } from 'vue'
import { useAuth } from '~/composables/useAuth'
import { useAccounts } from '~/composables/useAccounts'
import type { AccountType, AuthFlowState, ProviderConfig, ImapConnectionConfig, AccountSettings } from '~/types/sync'

const { t } = useI18n()

const { startOAuth2, storeImapCredentials, isAuthenticating, error: authError } = useAuth()
const { createAccount } = useAccounts()

// Provider configurations
const providers = computed<ProviderConfig[]>(() => [
  {
    id: 'gmail',
    name: t('pages.addAccount.providers.gmail.name'),
    icon: 'lucide:mail',
    description: t('pages.addAccount.providers.gmail.description'),
    auth_type: 'oauth',
    color: '#EA4335',
  },
  {
    id: 'office365',
    name: t('pages.addAccount.providers.office365.name'),
    icon: 'lucide:mail',
    description: t('pages.addAccount.providers.office365.description'),
    auth_type: 'oauth',
    color: '#0078D4',
  },
  {
    id: 'imap',
    name: t('pages.addAccount.providers.imap.name'),
    icon: 'lucide:server',
    description: t('pages.addAccount.providers.imap.description'),
    auth_type: 'basic',
    color: '#6B7280',
  },
  {
    id: 'apple',
    name: t('pages.addAccount.providers.apple.name'),
    icon: 'lucide:apple',
    description: t('pages.addAccount.providers.apple.description'),
    auth_type: 'basic',
    color: '#000000',
  },
])

// Auth flow state
const flowState = ref<AuthFlowState>({
  step: 'select',
})

// IMAP configuration
const imapConfig = ref<ImapConnectionConfig>({
  host: '',
  port: 993,
  username: '',
  password: '',
  use_tls: true,
})

// SMTP configuration (optional)
const smtpConfig = ref({
  host: '',
  port: 587,
  username: '',
  password: '',
  use_tls: true,
  use_different_credentials: false,
})

// Show advanced SMTP settings
const showSmtpSettings = ref(false)

// Account info
const accountName = ref('')
const accountEmail = ref('')

// Computed
const currentProvider = computed(() => {
  return providers.value.find(p => p.id === flowState.value.provider)
})

const isOAuthProvider = computed(() => {
  return currentProvider.value?.auth_type === 'oauth'
})

const canProceed = computed(() => {
  if (flowState.value.step === 'select') {
    return accountName.value && accountEmail.value && flowState.value.provider
  }
  if (flowState.value.step === 'configure' && !isOAuthProvider.value) {
    return imapConfig.value.host && imapConfig.value.username && imapConfig.value.password
  }
  return false
})

// Methods
const selectProvider = (providerId: AccountType) => {
  flowState.value.provider = providerId
}

const nextStep = async () => {
  if (flowState.value.step === 'select') {
    if (!flowState.value.provider) return

    // For OAuth providers, skip configuration and start auth immediately
    if (isOAuthProvider.value) {
      await startAuthentication()
    } else {
      flowState.value.step = 'configure'
    }
  } else if (flowState.value.step === 'configure') {
    await startAuthentication()
  }
}

const startAuthentication = async () => {
  if (!flowState.value.provider) return

  flowState.value.step = 'connecting'
  flowState.value.error = undefined

  try {
    // Prepare account settings
    const settings: AccountSettings = {
      imap_host: imapConfig.value.host || undefined,
      imap_port: imapConfig.value.port || undefined,
      imap_use_tls: imapConfig.value.use_tls,
      imap_username: imapConfig.value.username || undefined,
      // SMTP settings - if not provided, will fallback to IMAP settings on backend
      smtp_host: smtpConfig.value.host || undefined,
      smtp_port: smtpConfig.value.port || undefined,
      smtp_use_tls: smtpConfig.value.use_tls,
      smtp_username: smtpConfig.value.username || undefined,
      sync_enabled: true,
      sync_interval: 300,
      sync_on_startup: true,
      cache_attachments: true,
      max_attachment_cache_size: 1024 * 1024 * 1024, // 1GB
      auto_download_inline: true,
      provider_settings: undefined,
    }

    // Create account in database
    const accountId = await createAccount({
      name: accountName.value,
      email: accountEmail.value,
      account_type: flowState.value.provider,
      settings: !isOAuthProvider.value ? settings : undefined,
    })

    if (isOAuthProvider.value) {
      // Start OAuth flow
      await startOAuth2(flowState.value.provider, accountId)

      // Show waiting state for OAuth callback
      // The callback page will handle the token exchange
    } else {
      // Store IMAP credentials
      await storeImapCredentials(accountId, imapConfig.value)

      // Store SMTP credentials if different from IMAP
      if (smtpConfig.value.use_different_credentials && smtpConfig.value.password) {
        // Note: We store SMTP credentials as IMAP credentials in the credential store
        // The backend will use the same credentials for both IMAP and SMTP unless
        // a separate SMTP credential store is implemented
        // For now, SMTP password should be the same as IMAP or handled via the same credential
      }

      flowState.value.step = 'success'
      flowState.value.account_id = accountId
    }
  } catch (err) {
    flowState.value.step = 'error'
    flowState.value.error = err instanceof Error ? err.message : String(err)
  }
}

const goBack = () => {
  if (flowState.value.step === 'configure') {
    flowState.value.step = 'select'
  } else if (flowState.value.step === 'error') {
    flowState.value.step = flowState.value.provider && isOAuthProvider.value ? 'select' : 'configure'
    flowState.value.error = undefined
  }
}

const closeWindow = () => {
  // Close the auth window
  if (typeof window !== 'undefined') {
    window.close()
  }
}

const retry = () => {
  flowState.value.step = flowState.value.provider && isOAuthProvider.value ? 'select' : 'configure'
  flowState.value.error = undefined
}

// Auto-fill IMAP settings for Apple
const fillAppleSettings = () => {
  if (flowState.value.provider === 'apple' && accountEmail.value) {
    imapConfig.value.host = 'imap.mail.me.com'
    imapConfig.value.port = 993
    imapConfig.value.use_tls = true
    imapConfig.value.username = accountEmail.value
  }
}

definePageMeta({
  layout: 'empty'
})
</script>

<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4">
    <div class="w-full max-w-2xl">
      <!-- Header -->
      <div class="text-center mb-8">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
          {{
            flowState.step === 'select' ? t('pages.addAccount.steps.addEmail') :
              flowState.step === 'configure' ? t('pages.addAccount.steps.configure') :
                flowState.step === 'connecting' ? t('pages.addAccount.steps.authenticating') :
                  flowState.step === 'success' ? t('pages.addAccount.steps.success') : t('pages.addAccount.steps.error')
          }}
        </h1>
        <p
          v-if="flowState.step === 'select'"
          class="text-gray-600 dark:text-gray-400"
        >
          {{ t('pages.addAccount.descriptions.select') }}
        </p>
        <p
          v-else-if="flowState.step === 'configure'"
          class="text-gray-600 dark:text-gray-400"
        >
          {{ t('pages.addAccount.descriptions.configure') }}
        </p>
      </div>
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
        <!-- Step 1: Provider Selection -->
        <div
          v-if="flowState.step === 'select'"
          class="space-y-6"
        >
          <!-- Account Info -->
          <div class="space-y-4">
            <div>
              <label
                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                for="account-name"
              >
                {{ t('pages.addAccount.accountName.label') }}
              </label>
              <input
                id="account-name"
                v-model="accountName"
                :placeholder="t('pages.addAccount.accountName.placeholder')"
                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                type="text"
              >
            </div>

            <div>
              <label
                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                for="account-email"
              >
                {{ t('pages.addAccount.emailAddress.label') }}
              </label>
              <input
                id="account-email"
                v-model="accountEmail"
                :placeholder="t('pages.addAccount.emailAddress.placeholder')"
                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                type="email"
                @blur="fillAppleSettings"
              >
            </div>
          </div>

          <!-- Provider Selection -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
              {{ t('pages.addAccount.chooseProvider') }}
            </label>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <button
                v-for="provider in providers"
                :key="provider.id"
                :class="[
                  'p-4 border-2 rounded-lg text-left transition-all hover:shadow-md',
                  flowState.provider === provider.id
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'
                ]"
                @click="selectProvider(provider.id)"
              >
                <div class="flex items-start space-x-3">
                  <div
                    :class="flowState.provider === provider.id ? 'bg-white dark:bg-gray-800' : 'bg-gray-100 dark:bg-gray-700'"
                    :style="{ color: provider.color }"
                    class="flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center"
                  >
                    <Icon
                      :name="provider.icon"
                      class="w-6 h-6"
                    />
                  </div>
                  <div class="flex-1">
                    <h3 class="font-semibold text-gray-900 dark:text-white">
                      {{ provider.name }}
                    </h3>
                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      {{ provider.description }}
                    </p>
                  </div>
                </div>
              </button>
            </div>
          </div>
        </div>

        <!-- Step 2: IMAP Configuration -->
        <div
          v-else-if="flowState.step === 'configure'"
          class="space-y-4"
        >
          <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 mb-6">
            <div class="flex items-start space-x-3">
              <Icon
                class="w-5 h-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5"
                name="lucide:info"
              />
              <div class="text-sm text-blue-800 dark:text-blue-300">
                <p class="font-medium mb-1">{{ currentProvider?.name }} {{ t('pages.addAccount.descriptions.configure') }}</p>
                <p v-if="flowState.provider === 'apple'">
                  Use your iCloud email address and an app-specific password.
                  <a
                    class="underline"
                    href="https://support.apple.com/en-us/HT204397"
                    target="_blank"
                  >Learn how to
                    generate one</a>
                </p>
                <p v-else>
                  Enter your IMAP server details. Contact your email provider if you're unsure.
                </p>
              </div>
            </div>
          </div>

          <div>
            <label
              class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              for="imap-host"
            >
              {{ t('pages.addAccount.imap.server') }}
            </label>
            <input
              id="imap-host"
              v-model="imapConfig.host"
              class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="imap.example.com"
              type="text"
            >
          </div>

          <div>
            <label
              class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              for="imap-port"
            >
              {{ t('pages.addAccount.imap.port') }}
            </label>
            <input
              id="imap-port"
              v-model.number="imapConfig.port"
              class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              type="number"
            >
          </div>

          <div>
            <label
              class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              for="imap-username"
            >
              {{ t('pages.addAccount.imap.username') }}
            </label>
            <input
              id="imap-username"
              v-model="imapConfig.username"
              class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="username or email"
              type="text"
            >
          </div>

          <div>
            <label
              class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              for="imap-password"
            >
              {{ t('pages.addAccount.imap.password') }}
            </label>
            <input
              id="imap-password"
              v-model="imapConfig.password"
              class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              type="password"
            >
          </div>

          <div class="flex items-center space-x-2">
            <input
              id="use-tls"
              v-model="imapConfig.use_tls"
              class="w-4 h-4 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"
              type="checkbox"
            >
            <label
              class="text-sm text-gray-700 dark:text-gray-300"
              for="use-tls"
            >
              {{ t('pages.addAccount.imap.useTls') }}
            </label>
          </div>

          <!-- Advanced SMTP Settings -->
          <div class="pt-4 border-t border-gray-200 dark:border-gray-700">
            <button
              class="flex items-center justify-between w-full text-left"
              type="button"
              @click="showSmtpSettings = !showSmtpSettings"
            >
              <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {{ t('pages.addAccount.smtp.title') }}
              </span>
              <Icon
                :name="showSmtpSettings ? 'lucide:chevron-up' : 'lucide:chevron-down'"
                class="w-5 h-5 text-gray-500"
              />
            </button>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              {{ t('pages.addAccount.smtp.description') }}
            </p>

            <div
              v-if="showSmtpSettings"
              class="mt-4 space-y-4 pl-4 border-l-2 border-gray-200 dark:border-gray-700"
            >
              <div>
                <label
                  class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                  for="smtp-host"
                >
                  {{ t('pages.addAccount.smtp.server') }}
                </label>
                <input
                  id="smtp-host"
                  v-model="smtpConfig.host"
                  class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  placeholder="smtp.example.com (leave blank to use IMAP server)"
                  type="text"
                >
              </div>

              <div>
                <label
                  class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                  for="smtp-port"
                >
                  {{ t('pages.addAccount.smtp.port') }}
                </label>
                <input
                  id="smtp-port"
                  v-model.number="smtpConfig.port"
                  class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  placeholder="587"
                  type="number"
                >
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  {{ t('pages.addAccount.smtp.portHelper') }}
                </p>
              </div>

              <div>
                <label
                  class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                  for="smtp-username"
                >
                  {{ t('pages.addAccount.smtp.username') }}
                </label>
                <input
                  id="smtp-username"
                  v-model="smtpConfig.username"
                  class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  placeholder="Leave blank to use IMAP username"
                  type="text"
                >
              </div>

              <div class="flex items-center space-x-2">
                <input
                  id="smtp-use-tls"
                  v-model="smtpConfig.use_tls"
                  class="w-4 h-4 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"
                  type="checkbox"
                >
                <label
                  class="text-sm text-gray-700 dark:text-gray-300"
                  for="smtp-use-tls"
                >
                  {{ t('pages.addAccount.smtp.useTls') }}
                </label>
              </div>

              <div class="flex items-center space-x-2">
                <input
                  id="smtp-different-creds"
                  v-model="smtpConfig.use_different_credentials"
                  class="w-4 h-4 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"
                  type="checkbox"
                >
                <label
                  class="text-sm text-gray-700 dark:text-gray-300"
                  for="smtp-different-creds"
                >
                  {{ t('pages.addAccount.smtp.differentPassword') }}
                </label>
              </div>

              <div v-if="smtpConfig.use_different_credentials">
                <label
                  class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                  for="smtp-password"
                >
                  {{ t('pages.addAccount.smtp.password') }}
                </label>
                <input
                  id="smtp-password"
                  v-model="smtpConfig.password"
                  class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  type="password"
                >
              </div>
            </div>
          </div>
        </div>

        <!-- Step 3: Connecting -->
        <div
          v-else-if="flowState.step === 'connecting'"
          class="text-center py-12"
        >
          <div class="inline-block animate-spin rounded-full h-16 w-16 border-b-2 border-blue-600 mb-4"/>
          <p class="text-lg text-gray-700 dark:text-gray-300">
            {{ isOAuthProvider ? t('pages.addAccount.connecting.oauth') : t('pages.addAccount.connecting.testing') }}
          </p>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">
            {{ t('pages.addAccount.connecting.pleaseWait') }}
          </p>
        </div>

        <!-- Step 4: Success -->
        <div
          v-else-if="flowState.step === 'success'"
          class="text-center py-12"
        >
          <div
            class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-green-100 dark:bg-green-900/20 mb-4"
          >
            <Icon
              class="w-8 h-8 text-green-600 dark:text-green-400"
              name="lucide:check"
            />
          </div>
          <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
            {{ t('pages.addAccount.success.title') }}
          </h3>
          <p class="text-gray-600 dark:text-gray-400 mb-6">
            {{ t('pages.addAccount.success.message') }}
          </p>
        </div>

        <!-- Step 5: Error -->
        <div
          v-else-if="flowState.step === 'error'"
          class="py-12"
        >
          <div class="text-center mb-6">
            <div
              class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-red-100 dark:bg-red-900/20 mb-4"
            >
              <Icon
                class="w-8 h-8 text-red-600 dark:text-red-400"
                name="lucide:x"
              />
            </div>
            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
              {{ t('pages.addAccount.error.title') }}
            </h3>
          </div>

          <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
            <p class="text-sm text-red-800 dark:text-red-300">
              {{ flowState.error || authError || 'An unknown error occurred' }}
            </p>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-between mt-8 pt-6 border-t border-gray-200 dark:border-gray-700">
          <button
            v-if="flowState.step !== 'connecting' && flowState.step !== 'success'"
            :disabled="isAuthenticating"
            class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-50"
            @click="flowState.step === 'error' ? retry() : goBack()"
          >
            {{ flowState.step === 'error' ? t('common.actions.tryAgain') : t('common.actions.back') }}
          </button>
          <div v-else/>

          <div class="flex items-center space-x-3">
            <button
              v-if="flowState.step === 'success'"
              class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors"
              @click="closeWindow"
            >
              {{ t('common.actions.done') }}
            </button>
            <button
              v-else-if="flowState.step === 'error'"
              class="px-6 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-lg font-medium transition-colors"
              @click="closeWindow"
            >
              {{ t('common.actions.cancel') }}
            </button>
            <button
              v-else-if="flowState.step !== 'connecting'"
              :disabled="!canProceed || isAuthenticating"
              class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              @click="nextStep"
            >
              {{ flowState.step === 'configure' ? t('common.actions.connect') : t('common.actions.continue') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
