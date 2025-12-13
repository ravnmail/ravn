<script lang="ts" setup>

import { SimpleTooltip } from '~/components/ui/tooltip'
import type { SidebarSectionItem } from '~/composables/useSidebarNavigation'

const isExpanded = ref(true)
const router = useRouter()

defineProps<SidebarSectionItem>()

const handleItemClick = (item: { href?: string; click?: () => void }) => {
  if (item.click) {
    item.click()
  } else if (item.href) {
    router.push(item.href)
  }
}

</script>

<template>
  <div class="flex flex-col">
    <button
      v-if="title"
      class="text-xs font-semibold uppercase text-sidebar-item-text px-2 py-1 rounded flex items-center gap-1 hover:bg-sidebar-item-hover-background hover:text-sidebar-item-hover-foreground"
      @click="isExpanded = !isExpanded"
    >
      <span>{{ title }}</span>
      <Icon
        :name="`lucide:chevron-${isExpanded ? 'down' : 'up'}`"
      />
    </button>
    <div
      :class="[isExpanded ? 'block' : 'hidden', 'flex flex-col']">
      <SimpleTooltip
        v-for="item in children"
        :key="item.id"
        :tooltip-markdown="item.tooltip || item.name"
        side="right"
      >
        <button
          class="w-full text-left overflow-hidden pl-2 pr-1 py-1.5 flex items-center gap-1.5 rounded hover:bg-sidebar-item-hover-background"
          @click="() => handleItemClick(item)"
        >
          <Icon
            :name="`lucide:${item.icon || 'eye'}`"
            :size="18"
            :style="{ color: item.color }"
          />
          <span class="grow text-sm font-medium">{{ item.name }}</span>
        </button>
      </SimpleTooltip>
    </div>
    <slot/>
  </div>
</template>