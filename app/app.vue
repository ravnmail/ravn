<script lang="ts" setup>
import { useMagicKeys, whenever } from '@vueuse/core'
import { useQueryClient } from '@tanstack/vue-query'

import { Toaster } from 'vue-sonner'
import { AlertDialogProvider } from '@/composables/useAlertDialog'
import AddAccountModal from '~/components/Ravn/AddAccountModal.vue'
import ViewCreationWizard from '~/components/Ravn/ViewCreationWizard.vue'

const router = useRouter()
const queryClient = useQueryClient()
const isAddAccountModalOpen = ref(false)
const isCreateViewWizardOpen = ref(false)

useAppEvents()
useTheme()
useGlobalEventListeners(queryClient)

// Global keyboard shortcuts
const keys = useMagicKeys()
const cmdK = keys['Meta+K']

// Cmd+K / Ctrl+K to open search
whenever(cmdK, () => {
  router.push('/search')
})

provide('isAddAccountModalOpen', isAddAccountModalOpen)
provide('isCreateViewWizardOpen', isCreateViewWizardOpen)

</script>

<template>
  <div
    class="fixed top-0 left-0 w-full h-9 z-0"
    data-tauri-drag-region
  />
  <NuxtLayout>
    <CommandPalette/>
    <AlertDialogProvider class="h-screen">
      <NuxtPage/>
    </AlertDialogProvider>
    <ViewCreationWizard
      v-model:open="isCreateViewWizardOpen"
    />
    <AddAccountModal
      v-model:open="isAddAccountModalOpen"
    />
    <Toaster
      position="bottom-left"
      rich-colors
    />
  </NuxtLayout>
</template>
