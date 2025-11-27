<script lang="ts" setup>
import VerticalNavigationItem from '~/components/Ravn/VerticalNavigationItem.vue'

const { t } = useI18n()

const props = defineProps<{
  accountId: string
  accountName?: string
}>()

const { useNavigationFolders } = useFolders()
const items = useNavigationFolders(props.accountId)
const isExpanded = ref(true)

</script>

<template>
  <div class="flex flex-col">
    <button
      class="text-xs font-bold uppercase text-sidebar-item-text px-2 py-1 rounded flex items-center gap-1 hover:bg-sidebar-item-hover-background hover:text-sidebar-item-hover-foreground"
      @click="isExpanded = !isExpanded"
    >
      <span>{{ accountName || t('components.folderNav.mail') }}</span>
      <Icon
        :name="`lucide:chevron-${isExpanded ? 'down' : 'up'}`"
      />
    </button>
    <div :class="[isExpanded ? 'block' : 'hidden']">
      <div class="flex flex-col">
        <VerticalNavigationItem
          v-for="item in items"
          :key="item.id"
          v-bind="item"
        />
      </div>
      <div
        v-if="items.length === 0"
        class="p-4 text-center text-sidebar-item-text text-sm"
      >
        {{ t('components.folderNav.emptyState') }}
      </div>
    </div>
  </div>
</template>

