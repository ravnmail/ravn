<script lang="ts" setup>

import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  type EmptyMediaVariants,
  EmptyTitle
} from '~/components/ui/empty/index'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'

withDefaults(defineProps<{
  icon?: string
  iconName?: string
  variant?: EmptyMediaVariants["variant"]
  title?: string | CleanTranslation
  description?: string | CleanTranslation
}>(), {
  icon: undefined,
  iconName: undefined,
  variant: 'sticker',
  title: undefined,
  description: undefined
})

</script>

<template>
  <Empty>
    <EmptyHeader>
      <EmptyMedia
        v-if="iconName || icon"
        :variant="variant"
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