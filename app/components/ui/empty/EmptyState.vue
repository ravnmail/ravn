<script lang="ts" setup>

import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '~/components/ui/empty/index'

defineProps<{
  icon?: string
  iconName?: string
  title?: string
  description?: string
}>()

</script>

<template>

  <Empty>
    <EmptyHeader>
      <EmptyMedia variant="sticker">
        <Icon
          v-if="iconName"
          :name="iconName"
        />
        <span v-else-if="icon">{{ icon }}</span>
      </EmptyMedia>
      <EmptyTitle v-if="title">{{ title }}</EmptyTitle>
      <EmptyDescription v-if="description">{{ description }}</EmptyDescription>
    </EmptyHeader>
  </Empty>
</template>