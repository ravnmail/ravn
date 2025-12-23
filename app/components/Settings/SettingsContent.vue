<script lang="ts" setup>
import { ScrollArea } from '~/components/ui/scroll-area'
import SettingItem from '~/components/Settings/SettingItem.vue'
import type { SettingGroup } from '~/types/settings-manifest'
import EmptyState from '~/components/ui/empty/EmptyState.vue'

const { getGroup } = useSettingsManifest()

const props = defineProps<{
  groupId?: string | null
  search: string | null
  settingId?: string
}>()

const group = computed(() => props.groupId ? getGroup(props.groupId) as SettingGroup : null)
const { t } = useI18n()

</script>

<template>
  <ScrollArea
    class="h-full bg-background"
    direction="vertical"
  >
    <EmptyState
      v-if="!group"
      :description="t('settings.search.noResults.description', { search })"
      :title="t('settings.search.noResults.title')"
      class="h-full"
      icon="🛠️"
    />
    <div
      v-else
      class="space-y-3 py-8 px-10"
    >
      <h1 class="font-medium text-primary">
        {{ t(group.name) }}
      </h1>
      <div
        v-for="s in group.sections"
        :key="s.id"
      >
        <h2 class="font-semibold text-muted text-xs uppercase tracking-wide py-3">
          {{ t(s.name) }}
        </h2>
        <SettingItem
          v-for="item in s.items"
          :key="item.id"
          :group="group"
          :item="item"
          class="py-3 border-t border-border"
        />
      </div>
    </div>
  </ScrollArea>
</template>
