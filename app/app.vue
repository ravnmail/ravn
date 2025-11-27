<script lang="ts" setup>
import { useMagicKeys, whenever } from '@vueuse/core'

import { Toaster } from 'vue-sonner'
import { AlertDialogProvider } from '@/composables/useAlertDialog'

const router = useRouter()

useAppEvents()
useTheme()

// Global keyboard shortcuts
const keys = useMagicKeys()
const cmdK = keys['Meta+K']

// Cmd+K / Ctrl+K to open search
whenever(cmdK, () => {
  router.push('/search')
})
</script>

<template>
  <div
    class="fixed top-0 left-0 w-full h-10 z-0"
    data-tauri-drag-region
  />
  <NuxtLayout>
    <CommandPalette/>
    <AlertDialogProvider class="w-screen h-screen">
      <NuxtPage/>
    </AlertDialogProvider>
    <Toaster
      position="bottom-left"
      rich-colors
    />
  </NuxtLayout>
</template>
