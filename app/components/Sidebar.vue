<script lang="ts" setup>
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import Composer from '~/components/Composer.vue'
import { ScrollArea } from '~/components/ui/scroll-area'
import { Button } from '~/components/ui/button'
import { useStorage } from '@vueuse/core'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuSub,
  DropdownMenuSubContent, DropdownMenuSubTrigger,
  DropdownMenuTrigger
} from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import VerticalNavigationItem from '~/components/Ravn/VerticalNavigationItem.vue'
import { TreeItem, TreeRoot } from 'reka-ui'
import type { TreeItemToggleEvent } from 'reka-ui'
import type { SidebarNavigationItem, SidebarSectionItem } from '~/composables/useSidebarNavigation'

const { t } = useI18n()
const isAddAccountModalOpen = ref(false)
const showComposer = ref(false)

defineProps<{
  sticky?: boolean
  show?: boolean
}>()

const topItems = computed(() => {
  return [
    {
      id: 'composer',
      name: t('composer.composeNewEmail'),
      tooltip: t('composer.composeNewEmailTooltip'),
      icon: 'square-pen',
      click: () => {
        showComposer.value = true
      }
    },
    {
      id: 'home',
      name: t('home.title'),
      tooltip: t('home.tooltip'),
      icon: 'house',
      href: '/',
    },
    {
      id: 'search',
      name: t('search.title'),
      tooltip: t('search.tooltip'),
      icon: 'search',
      href: '/search',
    }
  ] as SidebarNavigationItem[]
})

const { sections } = useSidebarNavigation()
const expanded = useStorage<string[]>('sidebar', [])

const handleFolderExpandedChange = (folderId: string, e: boolean) => {
  if (e) {
    if (!expanded.value.includes(folderId)) {
      expanded.value.push(folderId)
    }
  } else {
    expanded.value = expanded.value.filter(id => id !== folderId)
  }
}

const onSelect = (item: SidebarNavigationItem | SidebarSectionItem, level: number) => {
  if (level === 1 || item.type === 'section') {
    return
  }

  if ('href' in item) {
    navigateTo(item.href)
  } else if ('click' in item && item.click) {
    item.click()
  }
}

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

const scrollContentRef = useTemplateRef('scrollContentRef')
const { bottom: scrollAreaHeight, top: scrollAreaTop } = useElementBounding(useTemplateRef('scrollArea'))
const { bottom: scrollContentHeight, top: scrollContentTop } = useElementBounding(scrollContentRef)

const needsBottomBorder = computed(() => {
  return scrollContentHeight.value > scrollAreaHeight.value
})

const needsTopBorder = computed(() => {
  return scrollAreaTop.value > scrollContentTop.value
})

const { Cmd_1 } = useMagicKeys()

watch(Cmd_1, (newVal) => {
  if (newVal) {
    const firstItem = (scrollContentRef.value?.querySelector('[aria-selected="true"]') ?? scrollContentRef.value?.querySelector('[data-reka-collection-item]')) as HTMLElement
    firstItem?.focus()
  }
})

const handleToggle = (e: TreeItemToggleEvent<SidebarNavigationItem>) => {
  if (e.detail.value?.type === 'folder' && e.detail.originalEvent instanceof PointerEvent) {
    e.preventDefault()
  }
}

</script>

<template>
  <nav
    :class="[sticky ? 'pt-12 h-screen bg-sidebar-background border-r' : 'fixed w-64 inset-y-24 rounded-r border-r border-t border-b left-0 pt-2 -translate-x-full transition-transform z-10', show ? 'translate-x-0' : '' ]"
    class="bg-sidebar-background flex flex-col pb-2 border-sidebar-border gap-2"
  >
    <div
      v-if="sticky"
      class="absolute top-0 left-0 w-full h-10 z-0"
      data-tauri-drag-region
    />
    <SidebarSection
      id="top-navigation"
      :children="topItems"
      class="px-2"
      type="section"
    />
    <ScrollArea
      ref="scrollArea"
      :class="['h-screen border-y border-transparent transition-border duration-300',
      needsBottomBorder && 'border-b-sidebar-border',
      needsTopBorder && 'border-t-sidebar-border'
      ]"
    >
      <div>
        <div
          ref="scrollContentRef"
          class="flex flex-col flex-1 gap-4 p-2"
        >
          <TreeRoot
            v-slot="{ flattenItems }"
            v-model:expanded="expanded"
            :get-key="({id}) => id"
            :items="sections"
          >
            <TreeItem
              v-for="(item, i) in flattenItems"
              :key="item._id"
              v-slot="{ isExpanded }"
              :class="[item.level === 1 && i > 0 ? 'mt-4' : '']"
              :style="{ 'padding-left': `${item.level - 2}rem` }"
              class="group mb-px bg-transparent overflow-clip focus:bg-selection focus:text-selection-foreground hover:bg-sidebar-item-hover-background rounded"
              v-bind="item.bind"
              @select="onSelect(item.value, item.level)"
              @toggle="(e) => handleToggle(e)"
            >
              <VerticalNavigationItem
                v-if="item.level > 1"
                :is-expanded="isExpanded"
                v-bind="item.value"
                @expanded="(v: boolean) => handleFolderExpandedChange(item.value.id, v)"
              />
              <button
                v-else-if="item.value.title"
                class="text-xs font-semibold uppercase text-sidebar-item-text px-2 py-1 rounded flex items-center gap-1 hover:bg-sidebar-item-hover-background hover:text-sidebar-item-hover-foreground"
              >
                <span>{{ item.value.title }}</span>
                <Icon
                  :class="isExpanded ? 'rotate-0' : 'rotate-180'"
                  :name="`lucide:chevron-down`"
                />
              </button>
            </TreeItem>
          </TreeRoot>
        </div>
      </div>
    </ScrollArea>
    <div class="pb-1 px-2">
      <DropdownMenu>
        <DropdownMenuTrigger>
          <Button
            class="p-1.5!"
            variant="ghost"
          >
            <Icon
              name="lucide:help-circle"
            />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="start"
          side="top"
        >
          <DropdownMenuItemRich
            icon="lucide:book-open"
            label="Documentation"
          />
          <DropdownMenuItemRich
            icon="lucide:messages-square"
            label="Get Support"
          />
          <DropdownMenuSub>
            <DropdownMenuSubTrigger>
              <Icon name="lucide:more-horizontal"/>
              <span>More</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent>
              <DropdownMenuItem>Rate Ravn</DropdownMenuItem>
              <DropdownMenuItem>View on GitHub</DropdownMenuItem>
              <DropdownMenuItem>Terms of Service</DropdownMenuItem>
              <DropdownMenuSeparator/>
              <DropdownMenuLabel>
                <p>RAVN Mail v25.12.12<br>Updated 11 hours ago</p>
              </DropdownMenuLabel>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSeparator/>
          <DropdownMenuLabel>What's new?</DropdownMenuLabel>
          <DropdownMenuGroup>
            <DropdownMenuItemRich
              icon="lucide:star"
              label="Rate Ravn"
            />
            <DropdownMenuItemRich
              icon="lucide:github"
              label="View on GitHub"
            />
          </DropdownMenuGroup>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
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

