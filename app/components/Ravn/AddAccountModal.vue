<script lang="ts" setup>
import { useAuth } from '~/composables/useAuth'
import type { AccountType, AuthFlowState, ProviderConfig, ImapConnectionConfig, AccountSettings } from '~/types/sync'

const { t } = useI18n()

const isDialogOpen = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{
  accountAdded: []
}>()

const { startOAuth2, storeImapCredentials, isAuthenticating, error: authError } = useAuth()
const { createAccount } = useAccounts()

const oauthTimeout = ref<ReturnType<typeof setTimeout> | null>(null)
const OAUTH_TIMEOUT_MS = 3 * 60 * 1000 // 3 minutes

const providers: ProviderConfig[] = [
  {
    id: 'gmail',
    name: 'Gmail',
    icon: 'lucide:mail',
    description: 'Connect your Gmail account',
    auth_type: 'oauth',
    color: '#EA4335',
  },
  {
    id: 'office365',
    name: 'Outlook / Office 365',
    icon: 'lucide:mail',
    description: 'Connect your Microsoft account',
    auth_type: 'oauth',
    color: '#0078D4',
  },
  {
    id: 'imap',
    name: 'IMAP',
    icon: 'lucide:server',
    description: 'Connect via IMAP protocol',
    auth_type: 'basic',
    color: '#6B7280',
  },
  {
    id: 'apple',
    name: 'Apple Mail',
    icon: 'lucide:apple',
    description: 'Connect your iCloud account',
    auth_type: 'basic',
    color: '#000000',
  },
]

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

const showSmtpSettings = ref(false)
const accountName = ref('')
const accountEmail = ref('')

const currentProvider = computed(() => {
  return providers.find(p => p.id === flowState.value.provider)
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

const dialogTitle = computed(() => {
  switch (flowState.value.step) {
    case 'select':
      return t('pages.addAccount.steps.addEmail')
    case 'configure':
      return t('pages.addAccount.steps.configure')
    case 'connecting':
      return t('pages.addAccount.steps.authenticating')
    case 'success':
      return t('pages.addAccount.steps.success')
    case 'error':
      return t('pages.addAccount.steps.error')
    default:
      return t('pages.addAccount.steps.addEmail')
  }
})

// Methods
const selectProvider = (providerId: AccountType) => {
  flowState.value.provider = providerId
}

const nextStep = async () => {
  if (flowState.value.step === 'select') {
    if (!flowState.value.provider) return

    if (isOAuthProvider.value) {
      await startAuthentication()
    } else {
      flowState.value.step = 'configure'
    }
  } else if (flowState.value.step === 'configure') {
    await startAuthentication()
  }
}

const cleanupOAuthFlow = () => {
  if (oauthTimeout.value) {
    clearTimeout(oauthTimeout.value)
    oauthTimeout.value = null
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

    const account = await createAccount({
      name: accountName.value,
      email: accountEmail.value,
      account_type: flowState.value.provider,
      settings: !isOAuthProvider.value ? settings : undefined,
    })

    if (isOAuthProvider.value) {
      await startOAuth2(flowState.value.provider, account.id)

      oauthTimeout.value = setTimeout(() => {
        cleanupOAuthFlow()
        flowState.value.step = 'error'
        flowState.value.error = 'Authentication timed out. Please try again.'
      }, OAUTH_TIMEOUT_MS)

    } else {
      await storeImapCredentials(account.id, imapConfig.value)

      flowState.value.step = 'success'
      flowState.value.account_id = account.id

      emit('accountAdded')
    }
  } catch (err) {
    cleanupOAuthFlow()
    flowState.value.step = 'error'
    flowState.value.error = err instanceof Error ? err.message : String(err)
  }
}

const cancelOAuth = () => {
  cleanupOAuthFlow()
  flowState.value.step = 'select'
  flowState.value.error = undefined
}

const goBack = () => {
  if (flowState.value.step === 'configure') {
    flowState.value.step = 'select'
  } else if (flowState.value.step === 'error') {
    flowState.value.step = flowState.value.provider && isOAuthProvider.value ? 'select' : 'configure'
    flowState.value.error = undefined
  }
}

const closeModal = () => {
  cleanupOAuthFlow()
  isDialogOpen.value = false
  // Reset form after a short delay to allow close animation
  setTimeout(resetForm, 300)
}

const resetForm = () => {
  flowState.value = { step: 'select' }
  accountName.value = ''
  accountEmail.value = ''
  imapConfig.value = {
    host: '',
    port: 993,
    username: '',
    password: '',
    use_tls: true,
  }
  smtpConfig.value = {
    host: '',
    port: 587,
    username: '',
    password: '',
    use_tls: true,
    use_different_credentials: false,
  }
  showSmtpSettings.value = false
}

const retry = () => {
  flowState.value.step = flowState.value.provider && isOAuthProvider.value ? 'select' : 'configure'
  flowState.value.error = undefined
}

const fillAppleSettings = () => {
  if (flowState.value.provider === 'apple' && accountEmail.value) {
    imapConfig.value.host = 'imap.mail.me.com'
    imapConfig.value.port = 993
    imapConfig.value.use_tls = true
    imapConfig.value.username = accountEmail.value
  }
}

const handleOAuthMessage = (event: MessageEvent) => {
  if (event.origin !== window.location.origin) return

  if (event.data.type === 'oauth-success') {
    cleanupOAuthFlow()
    flowState.value.step = 'success'
    flowState.value.error = undefined
    // Emit event to notify parent that account was added
    emit('accountAdded')
  } else if (event.data.type === 'oauth-error') {
    cleanupOAuthFlow()
    flowState.value.step = 'error'
    flowState.value.error = event.data.error || 'OAuth authentication failed'
  }
}

onMounted(() => {
  window.addEventListener('message', handleOAuthMessage)
})

onUnmounted(() => {
  cleanupOAuthFlow()
  window.removeEventListener('message', handleOAuthMessage)
})
</script>

<template>
  <UiDialog v-model:open="isDialogOpen">
    <UiDialogContent class="max-w-2xl max-h-[85vh] overflow-y-auto">
      <UiDialogHeader>
        <UiDialogTitle>{{ dialogTitle }}</UiDialogTitle>
        <UiDialogDescription v-if="flowState.step === 'select'">
          {{ $t('pages.addAccount.descriptions.select') }}
        </UiDialogDescription>
        <UiDialogDescription v-else-if="flowState.step === 'configure'">
          {{ $t('pages.addAccount.descriptions.configure') }}
        </UiDialogDescription>
      </UiDialogHeader>

      <div class="py-4">
        <!-- Step 1: Provider Selection -->
        <div
          v-if="flowState.step === 'select'"
          class="space-y-6"
        >
          <!-- Account Info -->
          <div class="space-y-4">
            <div class="space-y-2">
              <UiLabel for="account-name">
                {{ $t('pages.addAccount.accountName.label') }}
              </UiLabel>
              <UiInput
                id="account-name"
                v-model="accountName"
                :placeholder="$t('pages.addAccount.accountName.placeholder')"
                type="text"
              />
            </div>

            <div class="space-y-2">
              <UiLabel for="account-email">
                {{ $t('pages.addAccount.emailAddress.label') }}
              </UiLabel>
              <UiInput
                id="account-email"
                v-model="accountEmail"
                :placeholder="$t('pages.addAccount.emailAddress.placeholder')"
                type="email"
                @blur="fillAppleSettings"
              />
            </div>
          </div>

          <!-- Provider Selection -->
          <div class="space-y-3">
            <UiLabel>{{ $t('pages.addAccount.chooseProvider') }}</UiLabel>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <UiCard
                v-for="provider in providers"
                :key="provider.id"
                :class="[
                  'cursor-pointer transition-all hover:shadow-md',
                  flowState.provider === provider.id
                    ? 'ring-2 ring-primary'
                    : 'hover:border-primary/50'
                ]"
                @click="selectProvider(provider.id)"
              >
                <div class="p-4 flex items-start space-x-3">
                  <div
                    :style="{ color: provider.color }"
                    class="flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center bg-muted"
                  >
                    <Icon
                      :name="provider.icon"
                      class="w-6 h-6"
                    />
                  </div>
                  <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-sm">
                      {{ provider.name }}
                    </h3>
                    <p class="text-xs text-muted-foreground mt-1">
                      {{ provider.description }}
                    </p>
                  </div>
                </div>
              </UiCard>
            </div>
          </div>
        </div>

        <!-- Step 2: IMAP Configuration -->
        <div
          v-else-if="flowState.step === 'configure'"
          class="space-y-4"
        >
          <UiCard
            class="p-4"
            variant="accent"
          >
            <div class="flex items-start space-x-3">
              <Icon
                class="w-5 h-5 flex-shrink-0 mt-0.5"
                name="lucide:info"
              />
              <div class="text-sm">
                <p class="font-medium mb-1">{{ currentProvider?.name }} Configuration</p>
                <p
                  v-if="flowState.provider === 'apple'"
                  class="opacity-90"
                >
                  Use your iCloud email address and an app-specific password.
                  <a
                    class="underline hover:opacity-80"
                    href="https://support.apple.com/en-us/HT204397"
                    target="_blank"
                  >Learn how to generate one</a>
                </p>
                <p
                  v-else
                  class="opacity-90"
                >
                  Enter your IMAP server details. Contact your email provider if you're unsure.
                </p>
              </div>
            </div>
          </UiCard>

          <div class="space-y-2">
            <UiLabel for="imap-host">
              {{ $t('pages.addAccount.imap.server') }}
            </UiLabel>
            <UiInput
              id="imap-host"
              v-model="imapConfig.host"
              placeholder="imap.example.com"
              type="text"
            />
          </div>

          <div class="space-y-2">
            <UiLabel for="imap-port">
              {{ $t('pages.addAccount.imap.port') }}
            </UiLabel>
            <UiInput
              id="imap-port"
              v-model.number="imapConfig.port"
              type="number"
            />
          </div>

          <div class="space-y-2">
            <UiLabel for="imap-username">
              {{ $t('pages.addAccount.imap.username') }}
            </UiLabel>
            <UiInput
              id="imap-username"
              v-model="imapConfig.username"
              placeholder="username or email"
              type="text"
            />
          </div>

          <div class="space-y-2">
            <UiLabel for="imap-password">
              {{ $t('pages.addAccount.imap.password') }}
            </UiLabel>
            <UiInput
              id="imap-password"
              v-model="imapConfig.password"
              type="password"
            />
          </div>

          <div class="flex items-center space-x-2">
            <UiCheckbox
              id="use-tls"
              v-model:checked="imapConfig.use_tls"
            />
            <UiLabel
              class="cursor-pointer"
              for="use-tls"
            >
              {{ $t('pages.addAccount.imap.useTls') }} ({{ $t('pages.addAccount.tls.recommended') }})
            </UiLabel>
          </div>

          <!-- Advanced SMTP Settings -->
          <UiSeparator class="my-4"/>

          <div class="space-y-3">
            <UiButton
              class="w-full justify-between"
              type="button"
              variant="ghost"
              @click="showSmtpSettings = !showSmtpSettings"
            >
              <span class="text-sm font-medium">
                {{ $t('pages.addAccount.smtp.title') }}
              </span>
              <Icon
                :name="showSmtpSettings ? 'lucide:chevron-up' : 'lucide:chevron-down'"
                class="w-4 h-4"
              />
            </UiButton>
            <p class="text-xs text-muted-foreground">
              {{ $t('pages.addAccount.smtp.description') }}
            </p>

            <div
              v-if="showSmtpSettings"
              class="space-y-4 pl-4 border-l-2 mt-3"
            >
              <div class="space-y-2">
                <UiLabel for="smtp-host">
                  {{ $t('pages.addAccount.smtp.server') }}
                </UiLabel>
                <UiInput
                  id="smtp-host"
                  v-model="smtpConfig.host"
                  placeholder="smtp.example.com (leave blank to use IMAP server)"
                  type="text"
                />
              </div>

              <div class="space-y-2">
                <UiLabel for="smtp-port">
                  {{ $t('pages.addAccount.smtp.port') }}
                </UiLabel>
                <UiInput
                  id="smtp-port"
                  v-model.number="smtpConfig.port"
                  placeholder="587"
                  type="number"
                />
                <p class="text-xs text-muted-foreground">
                  {{ $t('pages.addAccount.smtp.portHelper') }}
                </p>
              </div>

              <div class="space-y-2">
                <UiLabel for="smtp-username">
                  {{ $t('pages.addAccount.smtp.username') }}
                </UiLabel>
                <UiInput
                  id="smtp-username"
                  v-model="smtpConfig.username"
                  placeholder="Leave blank to use IMAP username"
                  type="text"
                />
              </div>

              <div class="flex items-center space-x-2">
                <UiCheckbox
                  id="smtp-use-tls"
                  v-model:checked="smtpConfig.use_tls"
                />
                <UiLabel
                  class="cursor-pointer"
                  for="smtp-use-tls"
                >
                  {{ $t('pages.addAccount.smtp.useTls') }}
                </UiLabel>
              </div>

              <div class="flex items-center space-x-2">
                <UiCheckbox
                  id="smtp-different-creds"
                  v-model:checked="smtpConfig.use_different_credentials"
                />
                <UiLabel
                  class="cursor-pointer"
                  for="smtp-different-creds"
                >
                  {{ $t('pages.addAccount.smtp.differentPassword') }}
                </UiLabel>
              </div>

              <div
                v-if="smtpConfig.use_different_credentials"
                class="space-y-2"
              >
                <UiLabel for="smtp-password">
                  {{ $t('pages.addAccount.smtp.password') }}
                </UiLabel>
                <UiInput
                  id="smtp-password"
                  v-model="smtpConfig.password"
                  type="password"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Step 3: Connecting -->
        <div
          v-else-if="flowState.step === 'connecting'"
          class="text-center py-12"
        >
          <div class="inline-block animate-spin rounded-full h-16 w-16 border-b-2 border-primary mb-4"/>
          <p class="text-lg font-medium mb-2">
            {{ isOAuthProvider ? $t('pages.addAccount.connecting.oauth') : $t('pages.addAccount.connecting.testing') }}
          </p>

          <div
            v-if="isOAuthProvider"
            class="max-w-md mx-auto space-y-3"
          >
            <UiCard
              class="p-4"
              variant="accent"
            >
              <div class="flex items-start space-x-3 text-left">
                <Icon
                  class="w-5 h-5 flex-shrink-0 mt-0.5"
                  name="lucide:external-link"
                />
                <div class="text-sm">
                  <p class="font-medium mb-1">Complete authentication in new window</p>
                  <p class="opacity-90">
                    An authentication window has been opened. Please sign in with your {{ currentProvider?.name }}
                    account to continue.
                  </p>
                </div>
              </div>
            </UiCard>

            <p class="text-xs text-muted-foreground">
              The window will close automatically after successful authentication.
            </p>
          </div>

          <p
            v-else
            class="text-sm text-muted-foreground mt-2"
          >
            {{ $t('pages.addAccount.connecting.pleaseWait') }}
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
          <h3 class="text-xl font-semibold mb-2">
            {{ $t('pages.addAccount.success.title') }}
          </h3>
          <p class="text-muted-foreground">
            {{ $t('pages.addAccount.success.message') }}
          </p>
        </div>

        <!-- Step 5: Error -->
        <div
          v-else-if="flowState.step === 'error'"
          class="py-8"
        >
          <div class="text-center mb-6">
            <div
              class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-red-100 dark:bg-red-900/20 mb-4"
            >
              <Icon
                class="w-8 h-8 text-red-600 dark:text-red-400"
                name="lucide:alert-circle"
              />
            </div>
            <h3 class="text-xl font-semibold mb-2">
              {{ $t('pages.addAccount.error.title') }}
            </h3>
          </div>

          <UiCard
            class="p-4"
            variant="destructive"
          >
            <p class="text-sm">
              {{ flowState.error || authError || 'An unknown error occurred' }}
            </p>
          </UiCard>
        </div>
      </div>

      <UiDialogFooter>
        <div class="flex items-center justify-between w-full">
          <UiButton
            v-if="flowState.step === 'connecting' && isOAuthProvider"
            variant="ghost"
            @click="cancelOAuth"
          >
            {{ $t('common.actions.cancel') }}
          </UiButton>
          <UiButton
            v-else-if="flowState.step !== 'connecting' && flowState.step !== 'success'"
            :disabled="isAuthenticating"
            variant="ghost"
            @click="flowState.step === 'error' ? retry() : goBack()"
          >
            {{ flowState.step === 'error' ? $t('common.actions.tryAgain') : $t('common.actions.back') }}
          </UiButton>
          <div v-else/>

          <div class="flex items-center gap-3">
            <UiButton
              v-if="flowState.step === 'success'"
              @click="closeModal"
            >
              {{ $t('common.actions.done') }}
            </UiButton>
            <UiButton
              v-else-if="flowState.step === 'error'"
              variant="outline"
              @click="closeModal"
            >
              {{ $t('common.actions.cancel') }}
            </UiButton>
            <UiButton
              v-else-if="flowState.step !== 'connecting'"
              :disabled="!canProceed || isAuthenticating"
              @click="nextStep"
            >
              {{ flowState.step === 'configure' ? $t('common.actions.connect') : $t('common.actions.continue') }}
            </UiButton>
          </div>
        </div>
      </UiDialogFooter>
    </UiDialogContent>
  </UiDialog>
</template>
