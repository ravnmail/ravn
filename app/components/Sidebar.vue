<script lang="ts" setup>
import { check } from '@tauri-apps/plugin-updater'
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

import { toast } from 'vue-sonner'
import Composer from '~/components/Composer.vue'
import { ScrollArea } from '~/components/ui/scroll-area'
import { Button } from '~/components/ui/button'
import { useStorage } from '@vueuse/core'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem, DropdownMenuSeparator, DropdownMenuSub,
  DropdownMenuSubContent, DropdownMenuSubTrigger,
  DropdownMenuTrigger
} from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import VerticalNavigationItem from '~/components/Ravn/VerticalNavigationItem.vue'
import { TreeItem, TreeRoot } from 'reka-ui'
import type { TreeItemToggleEvent } from 'reka-ui'
import type { SidebarNavigationItem, SidebarSectionItem } from '~/composables/useSidebarNavigation'

import { version } from '~/../package.json'
import { invoke } from '@tauri-apps/api/core'

dayjs.extend(relativeTime)

const route = useRoute()
const showComposer = ref(false)

const lastUpdated = computed(() => {
  const buildDate = '2025-12-15T13:00:00Z'
  if (buildDate) {
    return dayjs(buildDate).fromNow()
  }
  return 'Unknown'
})

defineProps<{
  sticky?: boolean
  show?: boolean
}>()

const { register, unregister, executeAction, getAction } = useActions()

const topItems = computed(() => {
  return [
    getAction('global', 'composeEmail').value,
    getAction('global', 'home').value,
    getAction('global', 'search').value,
  ].filter(v => v !== undefined)
    .map(action => ({
      id: action.id,
      name: action.name,
      tooltip: action.tooltip,
      icon: action.icon,
      shortcut: action?.shortcut,
      click: () => {
        executeAction(action.key)
      }
    })) as SidebarNavigationItem[]
})

const { sections } = useSidebarNavigation()
const expanded = useStorage<string[]>('sidebar', [])

const selectedFolderId = computed(() => {
  return route.params.folder_id as string || route.params.view as string || null
})

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

onMounted(() => {
  register({
    id: 'gotoView',
    namespace: 'global',
    handler: (id: unknown) => {
      navigateTo(`/views/${id}`)
    }
  })
  register({
    id: 'gotoFolder',
    namespace: 'global',
    handler: (id: unknown) => {
      navigateTo(`/mail/null/folders/${id}`)
    }
  })
  register({
    id: 'composeEmail',
    namespace: 'global',
    icon: 'square-pen',
    handler: () => {
      showComposer.value = true
    }
  })
  register({
    id: 'search',
    namespace: 'global',
    icon: 'search',
    handler: () => {
      navigateTo('/search')
    }
  })
  register({
    id: 'home',
    namespace: 'global',
    icon: 'home',
    handler: () => {
      navigateTo('/')
    }
  })
  register({
    id: 'focusSidebarNavigation',
    namespace: 'global',
    handler: () => {
      const firstItem = (scrollContentRef.value?.querySelector('[aria-selected="true"]') ?? scrollContentRef.value?.querySelector('[data-reka-collection-item]')) as HTMLElement
      firstItem?.focus()
    }
  })
})

onBeforeUnmount(() => {
  unregister('global', 'gotoView')
  unregister('global', 'composeEmail')
  unregister('global', 'search')
  unregister('global', 'home')
  unregister('global', 'focusSidebarNavigation')
})

const handleToggle = (e: TreeItemToggleEvent<SidebarNavigationItem>) => {
  if (e.detail.value?.type === 'folder' && e.detail.originalEvent instanceof PointerEvent) {
    e.preventDefault()
  }
}

const openUrl = async (url: string) => {
  await invoke('open_external_url', { url })
}

const lastCheck = useStorage<string>('last_update_check', null)
const lastChecked = ref<string>('Never')
const lastCheckedInterval = ref<number | null>(null)

const updateLastChecked = () => {
  if (lastCheck.value) {
    lastChecked.value = dayjs(lastCheck.value).fromNow()
  } else {
    lastChecked.value = 'Never'
  }
}

watch(lastCheck, () => {
  updateLastChecked()
  clearInterval(lastCheckedInterval.value!)
  lastCheckedInterval.value = window.setInterval(() => {
    updateLastChecked()
  }, 60000)
}, { immediate: true })

onMounted(() => {
  lastCheckedInterval.value = window.setInterval(() => {
    updateLastChecked()
  }, 60000)
})

onBeforeUnmount(() => {
  if (lastCheckedInterval.value) {
    clearInterval(lastCheckedInterval.value)
  }
})

const checkForUpdate = async () => {
  try {
    lastCheck.value = new Date().toISOString()
    const update = await check()
    if (update) {
      toast.info('A new update is available! Downloading now...')
    } else {
      toast.info('You are already on the latest version.')
    }
  } catch (error) {
    console.error('Error checking for updates:', error)
    toast.error('Failed to check for updates.')
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
            :model-value="{ id: selectedFolderId }"
          >
            <TreeItem
              v-for="(item, i) in flattenItems"
              :key="item._id"
              v-slot="{ isExpanded }"
              :class="[item.level === 1 && i > 0 ? 'mt-4' : '']"
              :style="{ 'padding-left': `${item.level - 2}rem` }"
              class="group mb-px bg-transparent overflow-clip focus:bg-selection data-selected:bg-sidebar-item-hover-background data-selected:text-selection-foreground  focus:text-selection-foreground hover:bg-sidebar-item-hover-background rounded"
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
    <div class="pb-1 px-2 flex items-center gap-2">
      <DropdownMenu>
        <DropdownMenuTrigger>
          <Button
            size="xs"
            variant="ghost"
          >
            <Icon
              :size="16"
              name="lucide:help-circle"
            />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="start"
          side="top"
        >
          <DropdownMenuItemRich
            icon="lucide:keyboard"
            label="Keymap Editor"
            @select="() => navigateTo('/keymap-editor')"
          />
          <DropdownMenuItemRich
            icon="lucide:book-open"
            label="Documentation"
            @select="openUrl('https://www.ravnmail.com/docs')"
          />
          <DropdownMenuSeparator/>
          <DropdownMenuItemRich
            icon="lucide:lightbulb"
            label="Request a Feature"
            @select="openUrl('https://discord.gg/WWTfdpCwWE')"
          />
          <DropdownMenuItemRich
            icon="lucide:messages-square"
            label="Get Support"
            @select="openUrl('https://discord.gg/WWTfdpCwWE')"
          />
          <DropdownMenuSub>
            <DropdownMenuSubTrigger>
              <Icon name="lucide:more-horizontal"/>
              <span>More</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent>
              <DropdownMenuItem @select="openUrl('https://github.com/ravnmail/ravn')">View on GitHub</DropdownMenuItem>
              <DropdownMenuItem @select="openUrl('https://www.ravnmail.com/terms')">Terms of Service</DropdownMenuItem>
              <DropdownMenuSeparator/>
              <DropdownMenuItem
                class="text-xs font-semibold text-muted"
                @select="checkForUpdate"
              >
                <p>Ravn v{{ version }}<br>
                  Last Checked: {{ lastChecked }}
                </p>
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSeparator/>
          <DropdownMenuItemRich
            label="What's New"
            @select="openUrl(`https://www.ravnmail.com/release-notes#${version}`)"
          />
        </DropdownMenuContent>
      </DropdownMenu>
      <TrialBadge class="ml-auto"/>
    </div>
  </nav>
  <UnobstrusiveSheetContent
    v-if="showComposer"
    @close="onComposerSheetChange"
  >
    <Composer
      class="p-3"
      @discarded="handleComposerDiscarded"
      @sent="handleComposerSent"
    />
  </UnobstrusiveSheetContent>
</template>
