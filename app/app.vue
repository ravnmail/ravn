<script lang="ts" setup>
import { useQueryClient } from '@tanstack/vue-query'

import { Toaster } from 'vue-sonner'
import { AlertDialogProvider } from '@/composables/useAlertDialog'
import AddAccountModal from '~/components/Ravn/AddAccountModal.vue'
import ViewCreationWizard from '~/components/Ravn/ViewCreationWizard.vue'
import LicenseManagementDialog from '~/components/LicenseManagementDialog.vue'

const queryClient = useQueryClient()
const isAddAccountModalOpen = ref(false)
const isCreateViewWizardOpen = ref(false)
const isEnterLicenseDialogOpen = ref(false)

const { setupKeybindings, setupContext, register } = useActions()
const { getKeybindings, onKeybindingsChanged } = useKeybindings()
const { initPlatform } = usePlatform()

async function loadKeybindings() {
  try {
    const keybindings = await getKeybindings()
    setupKeybindings(keybindings)
  } catch (_) {
    setupKeybindings([])
  }
}

onMounted(async () => {
  initPlatform()
  setupContext([{ name: 'global', focused: computed(() => true) }])
  await loadKeybindings()

  register({
    id: 'openLicenseDialog',
    namespace: 'global',
    handler: () => {
      isEnterLicenseDialogOpen.value = true
    }
  })
  register({
    id: 'openAddAccountModal',
    namespace: 'global',
    handler: () => {
      isAddAccountModalOpen.value = true
    }
  })
  register({
    id: 'openCreateViewWizard',
    namespace: 'global',
    handler: () => {
      isCreateViewWizardOpen.value = true
    }
  })

  try {
    const unlisten = await onKeybindingsChanged(async () => {
      await loadKeybindings()
    })

    onBeforeUnmount(() => {
      unlisten()
      setupKeybindings([])
    })
  } catch (_) {
    // ignore
  }
})

onBeforeUnmount(() => {
  setupKeybindings([])
})

useAppEvents()
useTheme()
useGlobalEventListeners(queryClient)

// provide('isAddAccountModalOpen', isAddAccountModalOpen)
// provide('isCreateViewWizardOpen', isCreateViewWizardOpen)

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
    <LicenseManagementDialog
      v-model:open="isEnterLicenseDialogOpen"
    />
    <Toaster
      position="bottom-left"
      rich-colors
    />
  </NuxtLayout>
</template>
