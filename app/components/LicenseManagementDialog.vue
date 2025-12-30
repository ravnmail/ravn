<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import { Button } from '~/components/ui/button'
import { Alert, AlertDescription } from '~/components/ui/alert'
import { Badge } from '~/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '~/components/ui/dialog'
import { InputField } from '~/components/ui/form'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'success': []
}>()

const {
  licenseDetails,
  isLoading,
  activate,
  isTrialMode,
  daysRemaining,
  expirationStatus,
} = useLicense()

const licenseKey = ref('')
const resultMessage = ref('')
const resultType = ref<'success' | 'error' | null>(null)

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    resultMessage.value = ''
    resultType.value = null
    licenseKey.value = ''
  }
})

watch(licenseDetails, (newDetails) => {
  if (!newDetails?.is_trial) {
    licenseKey.value = newDetails?.key || ''
  }
})

const canSubmit = computed(() => {
  if (isLoading.value) return false

  return licenseKey.value.trim().length > 0
})


const handleSubmit = async () => {
  if (!canSubmit.value) return

  resultMessage.value = ''
  resultType.value = null

  const response = await activate(licenseKey.value.trim())

  if (response.success) {
    resultType.value = 'success'
    resultMessage.value = response.message

    setTimeout(() => {
      emit('success')
      emit('update:open', false)
    }, 1500)
  } else {
    resultType.value = 'error'
    resultMessage.value = response.message
  }
}

const openPricing = () => {
  invoke('open_external_url', { url: 'https://www.ravnmail.com/pricing' })
}

const getExpirationBadgeVariant = computed(() => {
  const status = expirationStatus.value
  if (status === 'expired') return 'destructive'
  if (status === 'expires_today' || status === 'expires_tomorrow') return 'destructive'
  if (status === 'expires_soon') return 'warning'
  return 'secondary'
})

const getExpirationMessage = computed(() => {
  const days = daysRemaining.value
  if (days === null || days === undefined) return null

  if (days < 0) return 'settings.license.status.expired'
  if (days === 0) return 'settings.license.status.expiresEndsToday'
  if (days === 1) return 'settings.license.status.expiresTomorrow'
  if (days <= 7) return 'settings.license.status.expiresSoon'
  return null
})
</script>

<template>
  <Dialog
    :open="open"
    @update:open="emit('update:open', $event)"
  >
    <DialogContent class="max-w-xl">
      <DialogHeader>
        <div class="flex items-start justify-between mr-5">
          <div class="flex-1">
            <DialogTitle>
              {{ $t(isTrialMode ? 'settings.license.upgradeTitle' : 'settings.license.manageTitle') }}
            </DialogTitle>
            <DialogDescription>
              {{ $t(isTrialMode ? 'settings.license.upgradeDescription' : 'settings.license.manageDescription') }}
            </DialogDescription>
          </div>
          <Badge
            v-if="isTrialMode && daysRemaining !== null"
            variant="info"
          >
            {{ $t('settings.license.daysRemaining', { days: daysRemaining}) }}
          </Badge>
        </div>
      </DialogHeader>
      <div class="flex flex-col gap-6 py-4">
        <Alert
          v-if="getExpirationMessage"
          :variant="getExpirationBadgeVariant === 'destructive' ? 'destructive' : 'warning'"
          class="mt-3"
        >
          <AlertDescription>
            {{ $t(getExpirationMessage, { days: daysRemaining }) }}
          </AlertDescription>
        </Alert>
        <Alert
          v-if="resultMessage"
          :variant="resultType === 'success' ? 'success' : 'destructive'"
        >
          <AlertDescription>
            {{ resultMessage }}
          </AlertDescription>
        </Alert>
        <InputField
          :label="$t('onboarding.license.emailLabel')"
          :model-value="licenseDetails?.user_email"
          disabled
          name="purchaseLicense"
        />
        <InputField
          v-model="licenseKey"
          :description="isTrialMode ? $t('onboarding.license.licenseKeyHint') : undefined"
          :disabled="isLoading || !isTrialMode"
          :label="$t('onboarding.license.licenseKeyLabel')"
          :placeholder="$t('onboarding.license.licenseKeyPlaceholder')"
          input-class="font-mono"
          name="licenseKey"
          @keyup.enter="handleSubmit"
        />
        <Alert
          v-if="isTrialMode"
          variant="info"
        >
          <h3 class="font-semibold">{{ $t('onboarding.license.noLicense.header') }}</h3>
          <AlertDescription class="flex flex-col gap-2">{{ $t('onboarding.license.noLicense.intro') }}</AlertDescription>
          <Button
            class="p-0"
            variant="link"
            @click="openPricing"
          >
            <span>{{ $t('onboarding.license.noLicense.cta') }}</span><Icon name="lucide:external-link"/>
          </Button>
        </Alert>
      </div>
      <DialogFooter class="gap-2">
        <Button
          :disabled="isLoading"
          variant="outline"
          @click="emit('update:open', false)"
        >
          {{ $t('common.actions.close') }}
        </Button>
        <Button
          v-if="isTrialMode"
          :disabled="!canSubmit"
          variant="primary"
          @click="handleSubmit"
        >
          <span
            v-if="isLoading"
            class="flex items-center gap-2"
          >
            <svg
              class="animate-spin h-4 w-4"
              fill="none"
              viewBox="0 0 24 24"
              xmlns="http://www.w3.org/2000/svg"
            >
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              />
              <path
                class="opacity-75"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                fill="currentColor"
              />
            </svg>
            {{ $t('onboarding.license.processing') }}
          </span>
          <span v-else>
            {{ $t('onboarding.license.activateLicense') }}
          </span>
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>