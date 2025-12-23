<script lang="ts" setup>
import { Input } from '~/components/ui/input'
import { TreeItem, type TreeItemToggleEvent, TreeRoot } from 'reka-ui'
import { ScrollArea } from '~/components/ui/scroll-area'
import type { SettingsTreeNode } from '~/types/settings-manifest'

const { t } = useI18n()

defineProps<{
  search: string | null
  modelValue: Record<string, unknown> | undefined
  navigation: SettingsTreeNode[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: Record<string, unknown> | undefined]
  'update:search': [value: string]
}>()

const expanded = ref<string[]>([])
const toggleExpanded = (groupId: string, e: boolean) => {
  if (e) {
    if (!expanded.value.includes(groupId)) {
      expanded.value.push(groupId)
    }
  } else {
    expanded.value = expanded.value.filter(id => id !== groupId)
  }
}

const handleToggle = (e: TreeItemToggleEvent<SettingsTreeNode>) => {
  if (e.detail.value?.children?.length && e.detail.originalEvent instanceof PointerEvent) {
    e.preventDefault()
  }
}

</script>

<template>
  <aside class="flex h-screen w-64 shrink-0 flex-col border-r border-border bg-surface">
    <div class="p-2 relative z-0">
      <Input
        :model-value="search"
        :placeholder="t('settings.searchPlaceholder')"
        class="w-full"
        type="search"
        @update:model-value="(v) => emit('update:search', v as string)"
      >
        <template #prefix>
          <Icon
            class="size-4 text-muted-foreground"
            name="lucide:search"
          />
        </template>
      </Input>
    </div>
    <ScrollArea
      class="flex-1 p-2"
      direction="vertical"
    >
      <TreeRoot
        v-slot="{ flattenItems }"
        v-model:expanded="expanded"
        :get-key="({id}) => id"
        :items="navigation"
        :model-value="modelValue"
        selection-behavior="replace"
        @update:model-value="(v) => emit('update:modelValue', v)"
      >
        <TreeItem
          v-for="item in flattenItems"
          :key="item._id"
          v-slot="{ isExpanded }"
          class="h-7 flex items-center group mb-px py-0.5 px-2 bg-transparent overflow-clip focus:bg-selection data-selected:bg-sidebar-item-hover-background data-selected:text-selection-foreground focus:text-selection-foreground hover:bg-sidebar-item-hover-background rounded"
          v-bind="item.bind"
          @toggle="(e) => handleToggle(e)"
        >
          <div
            class="w-6 flex opacity-50 hover:opacity-100 text-primary transition-opacity"
          >
            <Icon
            v-if="item.value.children?.length"
            :class="['transition-transform opacity-50', isExpanded ? 'transform rotate-90' : '']"
              name="lucide:chevron-right"
              @click.stop.prevent="toggleExpanded(item.value.id, !isExpanded)"
            />
          </div>
          <span class="text-sm font-medium truncate">{{ item.value.name }}</span>
        </TreeItem>
      </TreeRoot>
    </ScrollArea>
  </aside>
</template>
