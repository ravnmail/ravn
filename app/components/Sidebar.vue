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


const { bottom: scrollAreaHeight, top: scrollAreaTop } = useElementBounding(useTemplateRef('scrollArea'))
const { bottom: scrollContentHeight, top: scrollContentTop } = useElementBounding(useTemplateRef('scrollContentRef'))

const needsBottomBorder = computed(() => {
  return scrollContentHeight.value > scrollAreaHeight.value
})

const needsTopBorder = computed(() => {
  return scrollAreaTop.value > scrollContentTop.value
})

</script>

<template>
  <nav
    :class="[sticky ? 'pt-12 h-screen bg-sidebar-background border-r' : 'fixed w-64 inset-y-24 rounded-r border-r border-t border-b left-0 pt-2 -translate-x-full transition-transform z-10', show ? 'translate-x-0' : '' ]"
    class="bg-sidebar-background flex flex-col px-2 pb-2 border-sidebar-border overflow-hidden gap-2"
  >
    <div
      v-if="sticky"
      class="absolute top-0 left-0 w-full h-10 z-0"
      data-tauri-drag-region
    />
    <SidebarSection :items="topItems"/>
    <ScrollArea
      ref="scrollArea"
      :class="['h-screen -mx-1 px-1 border-y border-transparent mt-2 transition-border duration-300',
      needsBottomBorder ? 'border-b-sidebar-border' : '',
      needsTopBorder ? 'border-t-sidebar-border' : ''
      ]"
    >
      <div>
        <div
          ref="scrollContentRef"
          class="flex flex-col flex-1 gap-4"
        >
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

