<script lang="ts" setup>
// Enable keyboard shortcuts
import type { ReindexResult } from '~/composables/useSearch'
import { InputField } from '~/components/ui/form'
import { Button } from '~/components/ui/button'
import { invoke } from '@tauri-apps/api/core'

const isLoading = ref(false)
const { reindexAll } = useSearch()
const indexResult = ref<ReindexResult>({
  total_indexed: 0,
  success: null
})

const handleReindex = async () => {
  isLoading.value = true
  indexResult.value = await reindexAll()
  isLoading.value = false
}
const { testNotificationSound, updateBadgeCount } = useNotifications()

const soundName = ref('incoming_01')

</script>

<template>
  <div class="flex h-screen w-full flex-col gap-4 p-8">
    <div class="max-w-2xl space-y-6">
      <div class="border-t border-border pt-6 space-y-4">
        <h2 class="text-xl font-semibold">
          Other Tests
        </h2>

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

        <div class="space-y-2">
          <InputField
            v-model="soundName"
            label="Notification Sound"
            name="notificationSound"
          />
          <div class="flex gap-2">
            <Button @click="testNotificationSound(soundName)">
              Test Notification Sound
            </Button>
            <Button @click="updateBadgeCount(5)">
              Set Badge Count to 5
            </Button>
          </div>
        </div>

        <Button
          @click="invoke('resync_contact_counters')"
        >
          Resync Contact Counters
        </Button>
      </div>
    </div>
  </div>
</template>