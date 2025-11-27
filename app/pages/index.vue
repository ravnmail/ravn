<script lang="ts" setup>
// Enable keyboard shortcuts
import type { ReindexResult } from '~/composables/useSearch'
import { InputField } from '~/components/ui/form'
import { Button } from '~/components/ui/button'
import { toast } from 'vue-sonner'
import SelectField from '~/components/ui/form/SelectField.vue'
import {
  DropdownMenu,
  DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator,
  DropdownMenuGroup, DropdownMenuSub, DropdownMenuSubTrigger,
  DropdownMenuTrigger, DropdownMenuSubContent
} from '~/components/ui/dropdown-menu'
import FolderMenu from '~/components/Ravn/FolderMenu.vue'
import { DropdownMenuPortal } from 'reka-ui'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'

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

const testToast = (message: string) => {
  toast.error(message, {
    description: 'This is a persistent toast notification.',
    action: {
      label: 'Dismiss',
      onClick: (toastId) => toast.dismiss(toastId)
    },
    dismissible: true,
    duration: Infinity
  })
}

const { themes, currentTheme, switchTheme, isLoading: themeLoading } = useTheme()

const selectedTheme = ref<string>(currentTheme.value)

watch(currentTheme, (newTheme) => {
  selectedTheme.value = newTheme
})

const themeOptions = computed(() => {
  return themes.value.map(theme => ({
    value: theme.id,
    label: `${theme.name} (${theme.source})`
  }))
})

const handleThemeChange = async (themeId: string) => {
  try {
    await switchTheme(themeId)
    toast.success('Theme changed', {
      description: `Switched to ${themeId}`
    })
  } catch (err) {
    toast.error('Failed to change theme', {
      description: err instanceof Error ? err.message : String(err)
    })
  }
}

</script>

<template>
  <div class="flex h-screen w-full flex-col gap-4 p-8">
    <div class="max-w-2xl space-y-6">
      <div class="space-y-4">
        <h1 class="text-2xl font-bold">
          Theme Switcher
        </h1>

        <SelectField
          v-model="selectedTheme"
          :disabled="themeLoading"
          :options="themeOptions"
          label="Select Theme"
          name="theme"
          placeholder="Choose a theme"
          @update:model-value="handleThemeChange"
        />

        <div class="text-sm text-muted">
          Current theme: {{ currentTheme }}
        </div>
      </div>

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
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button class="mt-4">
                test dropdown
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
              <DropdownMenuGroup>
                <DropdownMenuItemRich
                  icon="lucide:reply"
                  label="Reply"
                  shortcut="R"
                />
                <DropdownMenuItemRich
                  icon="lucide:reply-all"
                  label="Reply All"
                  shortcut="R"
                />

                <DropdownMenuItemRich
                  icon="lucide:forward"
                  label="Forward"
                  shortcut="F"
                />

              </DropdownMenuGroup>
              <DropdownMenuSeparator/>
              <DropdownMenuGroup>
                <DropdownMenuItemRich
                  :shortcut="['Del']"
                  icon="lucide:archive"
                  label="Archive"
                />
                <DropdownMenuItemRich
                  icon="lucide:trash"
                  label="Delete"
                />
                <DropdownMenuSub>
                  <DropdownMenuSubTrigger>
                    <Icon name="lucide:folder-input"/>
                    <span>Move to...</span>
                    <DropdownMenuPortal>
                      <DropdownMenuSubContent>
                        <FolderMenu account-id="08092f47-cfd1-429f-8a29-ec58fa563a2a"/>
                      </DropdownMenuSubContent>
                    </DropdownMenuPortal>
                  </DropdownMenuSubTrigger>
                </DropdownMenuSub>
              </DropdownMenuGroup>
              <DropdownMenuSeparator/>
              <DropdownMenuGroup>
                <DropdownMenuItem>
                  Mark as unread
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
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
          @click="testToast('This is a test toast notification!')"
        >
          Show Test Toast
        </Button>
      </div>
    </div>
  </div>
</template>