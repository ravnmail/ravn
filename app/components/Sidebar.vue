<script lang="ts" setup>
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import Composer from '~/components/Composer.vue'
import { ScrollArea } from '~/components/ui/scroll-area'

const router = useRouter()
const { t } = useI18n()
const { accounts } = useAccounts()
const isAddAccountModalOpen = ref(false)
const searchQuery = ref('')
const showComposer = ref(false)

defineProps<{
  sticky?: boolean
  show?: boolean
}>()

const openAddAccountModal = () => {
  isAddAccountModalOpen.value = true
}

const handleSearch = () => {
  if (searchQuery.value.trim()) {
    router.push(`/search?q=${encodeURIComponent(searchQuery.value)}`)
  }
}

const handleSearchKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    handleSearch()
  }
}

const topItems = computed(() => {
  return [
    {
      id: 'composer',
      name: t('composer.composeNewEmail'),
      icon: 'square-pen',
      click: () => {
        showComposer.value = true
      }
    },
    {
      id: 'home',
      name: t('home.title'),
      icon: 'house',
      href: '/',
    },
    {
      id: 'search',
      name: t('search.title'),
      icon: 'search',
      href: '/search',
    }
  ]
})

const items = computed(() => {
  return [
    {
      id: 'settings',
      name: t('common.labels.settings'),
      icon: 'settings',
      href: '/settings',
    },
    // {
    //   id: 'add-account',
    //   icon: 'circle-question-mark',
    //   name: t('settings.accounts.addAccount'),
    //   click: openAddAccountModal
    // }
  ]
})

const onComposerSheetChange = (e) => {
  if (!e) {
    showComposer.value = false
  }
}

const handleComposerSent = async () => {
  showComposer.value = false
}

const handleComposerDiscarded = () => {
  showComposer.value = false
}

</script>

<template>
  <nav
    :class="[sticky ? 'pt-12 h-screen bg-sidebar-background border-r' : 'fixed inset-y-24 rounded-r border-r border-t border-b left-0 pt-2 -translate-x-full transition-transform z-10', show ? 'translate-x-0' : '' ]"
    class="min-w-64 w-64 bg-sidebar-background flex flex-col px-2 pb-2 gap-4 border-sidebar-border overflow-hidden"
  >
    <div
      v-if="sticky"
      class="absolute top-0 left-0 w-full h-10 z-0"
      data-tauri-drag-region
    />
    <SidebarSection :items="topItems"/>
    <ScrollArea class="h-screen -mx-1 px-1">
      <div class="flex flex-col flex-1 gap-4">
        <ViewNavigation/>
        <div
          v-for="account in accounts"
          :key="account.id"
          class="flex flex-col gap-1"
        >
          <FolderNavigation
            :account-id="account.id"
            :account-name="account.name"
          />
        </div>
      </div>
    </ScrollArea>
    <SidebarSection :items="items"/>
  </nav>

  <RavnAddAccountModal
    v-model:open="isAddAccountModalOpen"
  />

  <UnobstrusiveSheetContent
    v-if="showComposer"
    @close="onComposerSheetChange"
  >
    <Composer
      @discarded="handleComposerDiscarded"
      @sent="handleComposerSent"
    />
  </UnobstrusiveSheetContent>
</template>

