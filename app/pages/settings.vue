<script lang="ts" setup>
import type { SettingsNavItem } from '~/types/settings'

definePageMeta({
  layout: 'default',
})

const { t } = useI18n()

const settingsNav = computed<SettingsNavItem[]>(() => [
  {
    title: t('pages.settings.nav.aiAssistant'),
    name: 'settings-ai',
    icon: 'lucide:sparkles',
  },
  {
    title: t('pages.settings.nav.signatures'),
    name: 'settings-signatures',
    icon: 'lucide:pen-tool',
  },
])
</script>

<template>
  <div class="flex h-screen w-full">
    <!-- Sidebar Navigation -->
    <aside class="w-64 shrink-0 border-r border-border bg-muted/30">
      <div class="flex h-full flex-col">
        <!-- Header -->
        <div class="border-b border-border px-6 py-4">
          <h1 class="text-xl font-semibold">{{ t('common.labels.settings') }}</h1>
        </div>

        <!-- Navigation Items -->
        <ScrollArea class="flex-1 px-3 py-4">
          <nav class="space-y-1">
            <NuxtLink
              v-for="item in settingsNav"
              :key="item.name"
              :to="{ name: item.name }"
              class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
              exact-active-class="bg-accent text-accent-foreground"
            >
              <Icon
                :name="item.icon"
                class="size-5"
              />
              <span>{{ item.title }}</span>
              <Badge
                v-if="item.badge"
                class="ml-auto"
                variant="secondary"
              >
                {{ item.badge }}
              </Badge>
            </NuxtLink>
          </nav>
        </ScrollArea>
      </div>
    </aside>

    <!-- Content Area -->
    <main class="flex flex-1 flex-col overflow-hidden">
      <!-- Page Content (child routes) -->
      <ScrollArea class="flex-1">
        <div class="container max-w-4xl py-8">
          <NuxtPage/>
        </div>
      </ScrollArea>
    </main>
  </div>
</template>
