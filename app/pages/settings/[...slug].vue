<script lang="ts" setup>
import type { SettingsTreeNode } from '~/types/settings-manifest'

const { filter, getSelection, filteredNavigation } = useSettingsManifest()

const internalSelectedGroup = ref<SettingsTreeNode | null>(null)
const selectedGroup = computed<SettingsTreeNode | null>({
  get() {
    return getSelection(internalSelectedGroup.value?.groupId)
  },
  set(value) {
    internalSelectedGroup.value = value
  }
})

</script>

<template>
  <div class="flex h-screen w-full">
    <SettingsSidebar
      v-model="selectedGroup"
      v-model:search="filter"
      :navigation="filteredNavigation"
    />
    <main class="flex flex-1 flex-col overflow-hidden">
      <SettingsContent
        :group-id="selectedGroup?.groupId"
        :search="filter"
      />
    </main>
  </div>
</template>
