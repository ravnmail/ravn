<script lang="ts" setup>
import SidebarLabelItemComponent from '~/components/Ravn/SidebarLabelItem.vue'
import { SimpleTooltip } from '~/components/ui/tooltip'
import type {
  SidebarLabelItem,
  SidebarNavigationItem,
  SidebarSectionItem,
} from '~/composables/useSidebarNavigation'
import type { CreateLabelRequest } from '~/types/view'

const isExpanded = ref(true)
const router = useRouter()

const props = defineProps<SidebarSectionItem>()

const { t } = useI18n()
const { createLabel } = useLabels()

// ─── Labels section detection ─────────────────────────────────────────────────

const isLabelsSection = computed(() => props.id === 'labels')

// The real label children — everything except the new-label sentinel
const labelItems = computed(() =>
  isLabelsSection.value
    ? ((props.children?.filter((c) => c.type === 'label') as SidebarLabelItem[]) ?? [])
    : []
)

// ─── Inline label creation ────────────────────────────────────────────────────

const isCreating = ref(false)
const newLabelName = ref('')
const newLabelInputRef = useTemplateRef<HTMLInputElement>('newLabelInput')

const openCreateLabel = async () => {
  isCreating.value = true
  await nextTick()
  newLabelInputRef.value?.focus()
}

const cancelCreateLabel = () => {
  isCreating.value = false
  newLabelName.value = ''
}

const submitCreateLabel = async () => {
  const name = newLabelName.value.trim()
  if (!name) {
    cancelCreateLabel()
    return
  }
  try {
    await createLabel({ name } as CreateLabelRequest)
  } catch (err) {
    console.error('[SidebarSection] Failed to create label:', err)
  } finally {
    cancelCreateLabel()
  }
}

const handleCreateInputKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    e.preventDefault()
    submitCreateLabel()
  } else if (e.key === 'Escape') {
    cancelCreateLabel()
  }
}

// ─── Generic item click ───────────────────────────────────────────────────────

const handleItemClick = (item: SidebarNavigationItem) => {
  if ('click' in item && item.click) {
    item.click()
  } else if ('href' in item && item.href) {
    router.push(item.href)
  }
}
</script>

<template>
  <div class="flex flex-col">
    <!-- Section header (collapse toggle only — no + button) -->
    <button
      v-if="title"
      class="flex items-center gap-1 rounded px-2 py-1 text-xs font-semibold text-sidebar-item-text uppercase hover:bg-sidebar-item-hover-background hover:text-sidebar-item-hover-foreground"
      @click="isExpanded = !isExpanded"
    >
      <span>{{ title }}</span>
      <Icon :name="`lucide:chevron-${isExpanded ? 'down' : 'up'}`" />
    </button>

    <!-- Children list -->
    <div :class="[isExpanded ? 'flex flex-col' : 'hidden']">
      <!-- ── Labels section ──────────────────────────────────────────────── -->
      <template v-if="isLabelsSection">
        <!-- Existing labels -->
        <SidebarLabelItemComponent
          v-for="item in labelItems"
          :key="item.id"
          v-bind="item"
        />

        <!-- Inline input (shown after clicking the sentinel row) -->
        <div
          v-if="isCreating"
          class="flex items-center gap-1.5 rounded px-2 py-1.5"
        >
          <Icon
            class="shrink-0 text-muted"
            name="lucide:tag"
            :size="16"
          />
          <input
            ref="newLabelInput"
            v-model="newLabelName"
            :placeholder="t('components.sidebarNav.newLabelPlaceholder') as string"
            class="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted"
            @keydown="handleCreateInputKeydown"
            @blur="submitCreateLabel"
          />
        </div>

        <!-- "New Label" sentinel row — same style as "New View" -->
        <button
          v-else
          class="flex w-full items-center gap-1.5 overflow-hidden rounded py-1.5 pr-1 pl-2 text-left text-muted hover:bg-sidebar-item-hover-background hover:text-sidebar-item-hover-foreground"
          @click="openCreateLabel"
        >
          <Icon
            class="shrink-0"
            name="lucide:plus"
            :size="16"
          />
          <span class="grow text-sm font-medium">{{ t('components.sidebarNav.newLabel') }}</span>
        </button>
      </template>

      <!-- ── All other sections (views, folder accounts) ─────────────────── -->
      <template v-else>
        <SimpleTooltip
          v-for="item in children"
          :key="item.id"
          :shortcut="'shortcut' in item ? item.shortcut : undefined"
          :tooltip-markdown="
            (('tooltip' in item ? item.tooltip : undefined) || item.name) as string
          "
          side="right"
        >
          <button
            class="flex w-full items-center gap-1.5 overflow-hidden rounded py-1.5 pr-1 pl-2 text-left hover:bg-sidebar-item-hover-background"
            @click="handleItemClick(item)"
          >
            <Icon
              :name="`lucide:${item.icon || 'eye'}`"
              :size="18"
              :style="{ color: item.color }"
            />
            <span class="grow text-sm font-medium">{{ item.name }}</span>
          </button>
        </SimpleTooltip>
      </template>
    </div>

    <slot />
  </div>
</template>
