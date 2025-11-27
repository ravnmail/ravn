<script lang="ts" setup>
import { onMounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useAuth } from '~/composables/useAuth'

const { t } = useI18n()
const { exchangeOAuth2Code } = useAuth()

const status = ref<'processing' | 'success' | 'error'>('processing')
const message = ref('')
const errorDetail = ref('')

onMounted(async () => {
  try {
    // Get OAuth parameters from URL
    const params = new URLSearchParams(window.location.search)
    const code = params.get('code')
    const state = params.get('state')
    const error = params.get('error')
    const errorDescription = params.get('error_description')

    if (error) {
      status.value = 'error'
      message.value = t('pages.auth.callback.error.missingCode')
      errorDetail.value = errorDescription || error
      return
    }

    if (!code) {
      status.value = 'error'
      message.value = t('pages.auth.callback.error.missingCode')
      errorDetail.value = t('pages.auth.callback.error.missingCodeDetail')
      return
    }

    await exchangeOAuth2Code(code, state || undefined)

    status.value = 'success'
    message.value = t('pages.auth.callback.success.message')

    if (window.opener && !window.opener.closed) {
      window.opener.postMessage({ type: 'oauth-success' }, window.location.origin)
    }

    setTimeout(async () => {
      try {
        const currentWindow = getCurrentWindow()
        await currentWindow.close()
      } catch (e) {
        console.error('Failed to close window:', e)
        window.close()
      }
    }, 2000)
  } catch (err) {
    status.value = 'error'
    message.value = 'Token Exchange Failed'
    errorDetail.value = err instanceof Error ? err.message : String(err)

    if (window.opener && !window.opener.closed) {
      window.opener.postMessage({
        type: 'oauth-error',
        error: err instanceof Error ? err.message : String(err)
      }, window.location.origin)
    }
  }
})

const closeWindow = async () => {
  try {
    const currentWindow = getCurrentWindow()
    await currentWindow.close()
  } catch (e) {
    console.error('Failed to close window:', e)
    window.close()
  }
}
</script>

<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4">
    <div class="w-full max-w-md">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 text-center">
        <div
          v-if="status === 'processing'"
          class="py-8"
        >
          <div class="inline-block animate-spin rounded-full h-16 w-16 border-b-2 border-blue-600 mb-4"/>
          <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
            {{ t('pages.auth.callback.processing.title') }}
          </h3>
          <p class="text-gray-600 dark:text-gray-400">
            {{ t('pages.auth.callback.processing.message') }}
          </p>
        </div>
        <div
          v-else-if="status === 'success'"
          class="py-8"
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
            {{ message }}
          </h3>
          <p class="text-gray-600 dark:text-gray-400">
            {{ t('pages.auth.callback.success.autoClose') }}
          </p>
        </div>
        <div
          v-else
          class="py-8"
        >
          <div
            class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-red-100 dark:bg-red-900/20 mb-4"
          >
            <Icon
              class="w-8 h-8 text-red-600 dark:text-red-400"
              name="lucide:alert-circle"
            />
          </div>
          <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
            {{ message }}
          </h3>
          <div
            v-if="errorDetail"
            class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mt-4 mb-6"
          >
            <p class="text-sm text-red-800 dark:text-red-300">
              {{ errorDetail }}
            </p>
          </div>
          <button
            class="px-6 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-lg font-medium transition-colors"
            @click="closeWindow"
          >
            {{ t('pages.auth.callback.actions.closeWindow') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>