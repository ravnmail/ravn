<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'

import { Button } from '~/components/ui/button'
import { InputField } from '~/components/ui/form'
import type { ReindexResult } from '~/composables/useSearch'

const isLoading = ref(false)

const { reindexAll } = useSearch()
const { testNotificationSound, updateBadgeCount } = useNotifications()

const indexResult = ref<ReindexResult>({
  total_indexed: 0,
  success: false,
})

const zoomFactor = ref(1.0)
const soundName = ref('incoming_01')

const increaseZoom = () => {
  zoomFactor.value += 0.1
  getCurrentWebview().setZoom(zoomFactor.value)
}

const decreaseZoom = () => {
  zoomFactor.value = Math.max(0.2, zoomFactor.value - 0.1)
  getCurrentWebview().setZoom(zoomFactor.value)
}

const handleReindex = async () => {
  isLoading.value = true
  indexResult.value = await reindexAll()
  isLoading.value = false
}
</script>

<template>
  <div class="flex h-screen w-full flex-col gap-4 p-8">
    <div class="max-w-3xl space-y-6">
      <div class="space-y-1">
        <h1 class="text-2xl font-semibold">Debugging</h1>
        <p class="text-muted-foreground text-sm">
          Use these controls to validate local behaviors, notification flows, and background
          tooling.
        </p>
      </div>

      <div class="space-y-6 border-t border-border pt-6">
        <section class="space-y-3">
          <h2 class="text-xl font-semibold">Search</h2>
          <div>
            <Button
              :disabled="isLoading"
              @click="handleReindex"
            >
              Reindex
            </Button>
            <div class="mt-2 text-sm">
              {{ indexResult }}
            </div>
          </div>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-semibold">Window</h2>
          <div>
            <div class="mb-2 flex gap-2">
              <Button @click="decreaseZoom"> Zoom Out </Button>
              <Button @click="increaseZoom"> Zoom In </Button>
            </div>
            <div class="text-sm">Current Zoom: {{ (zoomFactor * 100).toFixed(0) }}%</div>
          </div>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-semibold">Notifications</h2>

          <div class="space-y-2">
            <InputField
              v-model="soundName"
              label="Notification Sound"
              name="notificationSound"
            />
            <div class="flex flex-wrap gap-2">
              <Button @click="testNotificationSound(soundName)"> Test Notification Sound </Button>
              <Button @click="updateBadgeCount()"> Refresh Badge Count </Button>
            </div>
          </div>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-semibold">Maintenance</h2>
          <Button @click="invoke('resync_contact_counters')"> Resync Contact Counters </Button>
        </section>
      </div>
    </div>
  </div>
</template>
