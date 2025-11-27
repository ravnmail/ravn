<script lang="ts" setup>
import ViewCreationWizard from '~/components/Ravn/ViewCreationWizard.vue'
import SidebarSection from '~/components/SidebarSection.vue'

const { t } = useI18n()
const { views } = useViews()

const isWizardOpen = ref(false)

const handleCreateView = () => {
  isWizardOpen.value = true
}
const handleViewCreated = () => {
  isWizardOpen.value = false
}

const items = computed(() => {
  const result = views.value.map(view => ({
    id: view.id,
    name: view.name,
    icon: view.icon || 'grid-3x3',
    color: view.color,
    href: `/views/${view.id}`,
  }))

  result.push({
    id: 'new-view',
    name: t('components.viewNav.newView') as string,
    icon: 'plus',
    click: handleCreateView,
  })

  return result
})


</script>

<template>
  <SidebarSection
    :items="items"
    :title="t('components.viewNav.title')"
  >
    <ViewCreationWizard
      v-model:open="isWizardOpen"
      @created="handleViewCreated"
    />
  </SidebarSection>
</template>

