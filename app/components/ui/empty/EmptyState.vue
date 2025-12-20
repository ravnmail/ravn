<script lang="ts" setup>

import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '~/components/ui/empty/index'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'

defineProps<{
  icon?: string
  iconName?: string
  title?: string | CleanTranslation
  description?: string | CleanTranslation
}>()

</script>

<template>
  <Empty>
    <EmptyHeader>
      <EmptyMedia
        v-if="iconName || icon"
        variant="sticker"
      >
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