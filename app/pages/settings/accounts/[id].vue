<script lang="ts" setup>
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { toast } from 'vue-sonner'
import { useAccounts } from '~/composables/useAccounts'
import { useAuth } from '~/composables/useAuth'
import type { Account, ImapConnectionConfig } from '~/types/sync'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const { getAccounts } = useAccounts()
const { storeImapCredentials } = useAuth()

const account = ref<Account | null>(null)
const isLoading = ref(false)
const isSaving = ref(false)

// Form state
const imapConfig = ref<ImapConnectionConfig>({
  host: '',
  port: 993,
  username: '',
  password: '',
  use_tls: true,
})

const smtpConfig = ref({
  host: '',
  port: 587,
  username: '',
  password: '',
  use_tls: true,
})

const showPassword = ref(false)

// Computed
const isImap = computed(() => account.value?.account_type === 'imap' || account.value?.account_type === 'apple')
const canSave = computed(() => {
  return imapConfig.value.host && imapConfig.value.username && imapConfig.value.password
})

// Methods
const loadAccount = async () => {
  try {
    isLoading.value = true
    const accountId = route.params.id as string
    const accounts = await getAccounts()
    const found = accounts.find(a => a.id === accountId)

    if (!found) {
      toast.error(t('common.states.noContent'))
      router.back()
      return
    }

    account.value = found

    // Populate form with existing settings
    if (found.settings) {
      imapConfig.value = {
        host: found.settings.imap_host || '',
        port: found.settings.imap_port || 993,
        username: found.settings.imap_username || '',
        password: '', // Don't pre-fill password for security
        use_tls: found.settings.imap_use_tls !== false,
      }

      smtpConfig.value = {
        host: found.settings.smtp_host || '',
        port: found.settings.smtp_port || 587,
        username: found.settings.smtp_username || '',
        password: '', // Don't pre-fill password for security
        use_tls: found.settings.smtp_use_tls !== false,
      }
    }
  } catch (err) {
    console.error('[AccountSettings] Failed to load account:', err)
    toast.error(t('common.states.processing'))
  } finally {
    isLoading.value = false
  }
}

const saveCredentials = async () => {
  if (!account.value) return

  try {
    isSaving.value = true

    // Store IMAP credentials
    if (isImap.value) {
      await storeImapCredentials(Number.parseInt(account.value.id), imapConfig.value)

      toast.success(t('pages.addAccount.success.message'), {
        description: t('pages.addAccount.success.title'),
      })

      // Navigate back to mail view
      router.push(`/mail/${account.value.id}`)
    }
  } catch (err) {
    console.error('[AccountSettings] Failed to save credentials:', err)
    const errorMessage = err instanceof Error ? err.message : String(err)
    toast.error(t('pages.addAccount.error.title'), {
      description: errorMessage,
    })
  } finally {
    isSaving.value = false
  }
}

const goBack = () => {
  router.back()
}

onMounted(() => {
  loadAccount()
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="space-y-2">
      <div class="flex items-center gap-2">
        <button
          class="inline-flex items-center justify-center rounded-md hover:bg-accent"
          @click="goBack"
        >
          <Icon
            class="size-5"
            name="lucide:arrow-left"
          />
        </button>
        <h1 class="text-2xl font-bold">{{ t('common.actions.edit') }} {{ account?.name || account?.email }}</h1>
      </div>
      <p class="text-sm text-muted-foreground">
        {{ account?.email }}
      </p>
    </div>

    <!-- Loading State -->
    <div
      v-if="isLoading"
      class="flex items-center justify-center py-8"
    >
      <Icon
        class="size-6 animate-spin"
        name="lucide:loader-2"
      />
    </div>

    <!-- Form Content -->
    <template v-else-if="account && isImap">
      <div class="space-y-6">
        <!-- IMAP Settings -->
        <div class="space-y-4 rounded-lg border border-border p-6">
          <h2 class="text-lg font-semibold">{{ t('pages.addAccount.imap.server') }}</h2>

          <!-- Host -->
          <div class="space-y-2">
            <label class="text-sm font-medium">{{ t('pages.addAccount.imap.server') }}</label>
            <input
              v-model="imapConfig.host"
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-sm"
              placeholder="imap.gmail.com"
              type="text"
            >
          </div>

          <!-- Port -->
          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <label class="text-sm font-medium">{{ t('pages.addAccount.imap.port') }}</label>
              <input
                v-model.number="imapConfig.port"
                class="w-full rounded-md border border-border bg-background px-3 py-2 text-sm"
                placeholder="993"
                type="number"
              >
            </div>

            <!-- TLS -->
            <div class="space-y-2">
              <label class="text-sm font-medium">{{ t('pages.addAccount.imap.useTls') }}</label>
              <div class="flex items-center gap-2 rounded-md border border-border bg-background px-3 py-2">
                <input
                  v-model="imapConfig.use_tls"
                  class="rounded border border-border"
                  type="checkbox"
                >
                <span class="text-xs text-muted-foreground">{{ t('pages.addAccount.tls.recommended') }}</span>
              </div>
            </div>
          </div>

          <!-- Username -->
          <div class="space-y-2">
            <label class="text-sm font-medium">{{ t('pages.addAccount.imap.username') }}</label>
            <input
              v-model="imapConfig.username"
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-sm"
              placeholder="user@example.com"
              type="text"
            >
          </div>

          <!-- Password -->
          <div class="space-y-2">
            <label class="text-sm font-medium">{{ t('pages.addAccount.imap.password') }}</label>
            <div class="relative">
              <input
                v-model="imapConfig.password"
                :type="showPassword ? 'text' : 'password'"
                class="w-full rounded-md border border-border bg-background px-3 py-2 text-sm pr-10"
              >
              <button
                class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                @click="showPassword = !showPassword"
              >
                <Icon
                  :name="showPassword ? 'lucide:eye-off' : 'lucide:eye'"
                  class="size-4"
                />
              </button>
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex gap-2">
          <button
            class="flex-1 rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-accent"
            @click="goBack"
          >
            {{ t('common.actions.cancel') }}
          </button>
          <button
            :disabled="!canSave || isSaving"
            class="flex-1 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            @click="saveCredentials"
          >
            <Icon
              v-if="isSaving"
              class="mr-2 inline size-4 animate-spin"
              name="lucide:loader-2"
            />
            {{ t('common.actions.save') }}
          </button>
        </div>
      </div>
    </template>

    <!-- Unsupported Provider -->
    <template v-else>
      <div class="rounded-lg border border-border bg-muted/50 p-6 text-center">
        <p class="text-sm text-muted-foreground">
          {{ t('common.states.noContent') }}
        </p>
      </div>
    </template>
  </div>
</template>
