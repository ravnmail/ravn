<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import { check } from '@tauri-apps/plugin-updater'
import { useStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import type { TreeItemToggleEvent } from 'reka-ui'
import { TreeItem, TreeRoot } from 'reka-ui'
import { toast } from 'vue-sonner'

import { version } from '~/../package.json'
import Composer from '~/components/Composer.vue'
import VerticalNavigationItem from '~/components/Ravn/VerticalNavigationItem.vue'
import SidebarSection from '~/components/SidebarSection.vue'
import { Button } from '~/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import { ScrollArea } from '~/components/ui/scroll-area'
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import type { SidebarNavigationItem, SidebarSectionItem } from '~/composables/useSidebarNavigation'

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
  ]
    .filter((v) => v !== undefined)
    .map((action) => ({
      id: action.id,
      name: action.name,
      tooltip: action.tooltip,
      icon: action.icon,
      shortcut: action?.shortcut,
      click: () => {
        executeAction(action.key)
      },
    })) as SidebarNavigationItem[]
})

const { sections } = useSidebarNavigation()
const expanded = useStorage<string[]>('sidebar', [])

/** The labels section is rendered separately outside the TreeRoot */
const labelsSection = computed<SidebarSectionItem | null>(
  () => (sections.value.find((s) => s.id === 'labels') as SidebarSectionItem) ?? null
)

/** Everything except the labels section goes into the TreeRoot */
const treeSections = computed(() => sections.value.filter((s) => s.id !== 'labels'))

const selectedFolderId = computed(() => {
  return (route.params.folder_id as string) || (route.params.view as string) || null
})

const handleFolderExpandedChange = (folderId: string, e: boolean) => {
  if (e) {
    if (!expanded.value.includes(folderId)) {
      expanded.value.push(folderId)
    }
  } else {
    expanded.value = expanded.value.filter((id) => id !== folderId)
  }
}

const onSelect = (item: SidebarNavigationItem | SidebarSectionItem, level: number) => {
  if (level === 1 || item.type === 'section') {
    return
  }

  if (item.type === 'label') {
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
const { bottom: scrollAreaHeight, top: scrollAreaTop } = useElementBounding(
  useTemplateRef('scrollArea')
)
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
    },
  })
  register({
    id: 'gotoFolder',
    namespace: 'global',
    handler: (id: unknown) => {
      navigateTo(`/mail/null/folders/${id}`)
    },
  })
  register({
    id: 'composeEmail',
    namespace: 'global',
    icon: 'square-pen',
    handler: () => {
      showComposer.value = true
    },
  })
  register({
    id: 'search',
    namespace: 'global',
    icon: 'search',
    handler: () => {
      navigateTo('/search')
    },
  })
  register({
    id: 'home',
    namespace: 'global',
    icon: 'home',
    handler: () => {
      navigateTo('/')
    },
  })
  register({
    id: 'focusSidebarNavigation',
    namespace: 'global',
    handler: () => {
      const firstItem = (scrollContentRef.value?.querySelector('[aria-selected="true"]') ??
        scrollContentRef.value?.querySelector('[data-reka-collection-item]')) as HTMLElement
      firstItem?.focus()
    },
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

watch(
  lastCheck,
  () => {
    updateLastChecked()
    clearInterval(lastCheckedInterval.value!)
    lastCheckedInterval.value = window.setInterval(() => {
      updateLastChecked()
    }, 60000)
  },
  { immediate: true }
)

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
    :class="[
      sticky
        ? 'h-screen border-r bg-sidebar-background pt-12'
        : 'fixed inset-y-24 left-0 z-10 w-64 -translate-x-full rounded-r border-t border-r border-b pt-2 transition-transform',
      show ? 'translate-x-0' : '',
    ]"
    class="flex flex-col gap-2 border-sidebar-border bg-sidebar-background pb-2"
  >
    <div
      v-if="sticky"
      class="absolute top-0 left-0 z-0 h-10 w-full"
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
      :class="[
        'transition-border h-screen border-y border-transparent duration-300',
        needsBottomBorder && 'border-b-sidebar-border',
        needsTopBorder && 'border-t-sidebar-border',
      ]"
    >
      <div>
        <div
          ref="scrollContentRef"
          class="flex flex-1 flex-col gap-4 p-2"
        >
          <TreeRoot
            v-slot="{ flattenItems }"
            v-model:expanded="expanded"
            :get-key="({ id }) => id"
            :items="treeSections"
            :model-value="{ id: selectedFolderId }"
          >
            <TreeItem
              v-for="(item, i) in flattenItems"
              :key="item._id"
              v-slot="{ isExpanded }"
              :class="[item.level === 1 && i > 0 ? 'mt-4' : '']"
              :style="{ 'padding-left': `${item.level - 2}rem` }"
              class="group mb-px overflow-clip rounded bg-transparent hover:bg-sidebar-item-hover-background focus:bg-selection focus:text-selection-foreground data-selected:bg-sidebar-item-hover-background data-selected:text-selection-foreground"
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
                class="flex items-center gap-1 rounded px-2 py-1 text-xs font-semibold text-sidebar-item-text uppercase hover:bg-sidebar-item-hover-background hover:text-sidebar-item-hover-foreground"
              >
                <span>{{ item.value.title }}</span>
                <Icon
                  :class="isExpanded ? 'rotate-0' : 'rotate-180'"
                  :name="`lucide:chevron-down`"
                />
              </button>
            </TreeItem>
          </TreeRoot>

          <!-- Labels section — rendered outside TreeRoot so SidebarLabelItem can use
               its own drop targets and dropdown menus without overflow-clip interference -->
          <SidebarSection
            v-if="labelsSection"
            v-bind="labelsSection"
          />
        </div>
      </div>
    </ScrollArea>
    <div class="flex items-center gap-2 px-2 pb-1">
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
          <DropdownMenuSeparator />
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
              <Icon name="lucide:more-horizontal" />
              <span>More</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent>
              <DropdownMenuItem @select="openUrl('https://github.com/ravnmail/ravn')"
                >View on GitHub</DropdownMenuItem
              >
              <DropdownMenuItem @select="openUrl('https://www.ravnmail.com/terms')"
                >Terms of Service</DropdownMenuItem
              >
              <DropdownMenuSeparator />
              <DropdownMenuItem
                class="text-xs font-semibold text-muted"
                @select="checkForUpdate"
              >
                <p>
                  Ravn v{{ version }}<br />
                  Last Checked: {{ lastChecked }}
                </p>
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSeparator />
          <DropdownMenuItemRich
            label="What's New"
            @select="openUrl(`https://www.ravnmail.com/release-notes#${version}`)"
          />
        </DropdownMenuContent>
      </DropdownMenu>
      <TrialBadge class="ml-auto" />
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
