<script lang="ts" setup>
import type { EmailListItem } from '~/types/email'
import { ScrollArea } from '~/components/ui/scroll-area'
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import EmailViewer from '~/components/Ravn/EmailViewer.vue'
import EmailListItemComponent from '~/components/Ravn/EmailListItem.vue'
import TantivySearchFilter from '~/components/Ravn/TantivySearchFilter.vue'
import { useTantivySearch, type SearchFields } from '~/composables/useTantivySearch'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const searchFields: SearchFields = {
  from: {
    type: 'text',
    label: t('labels.email.from'),
    description: 'Sender email address or name',
  },
  to: {
    type: 'text',
    label: t('labels.email.to'),
    description: 'Recipient email address or name',
  },
  cc: {
    type: 'text',
    label: t('labels.email.cc'),
    description: 'CC recipient email address or name',
  },
  subject: {
    type: 'text',
    label: t('labels.email.subject'),
    description: 'Email subject line',
  },
  body: {
    type: 'text',
    label: t('labels.email.body'),
    description: 'Email body content',
  },
  is_read: {
    type: 'boolean',
    label: t('labels.email.isRead'),
    description: 'Read/Unread status',
  },
  received_at: {
    type: 'datetime',
    label: t('labels.email.receivedAt'),
    description: 'Email received date/time',
  },
}
const { loading, results, totalResults, search } = useTantivySearch(searchFields)
const searchQuery = computed({
  get() {
    return route.query.q as string | undefined
  },
  set(value: string | undefined) {
    const query = { ...route.query }
    if (value) {
      query.q = value
    } else {
      delete query.q
    }
    router.replace({ query })
  },
})

watch(searchQuery, (newQuery) => {
  if (newQuery && newQuery.trim() !== '') {
    performSearch()
  }
}, { immediate: true })
const performSearch = async () => {
  if (searchQuery.value) {
    await search({ query: searchQuery.value, limit: 200 })
  }
}
const handleSearch = (query: string) => {
  searchQuery.value = query
}
const selectEmail = (email: EmailListItem) => {
  router.replace({ query: { ...route.query, email: email.id } })
}
const selectedEmailIdFromRoute = computed({
  get() {
    return route.query.email as string | undefined
  },
  set(value: string | undefined) {
    const query = { ...route.query }
    if (value) {
      query.email = value
    } else {
      delete query.email
    }
    router.replace({ query })
  },
})
const onSheetChange = (open: boolean) => {
  if (!open) {
    selectedEmailIdFromRoute.value = undefined
  }
}

useHead({
  title: computed(() => t('search.pageTitle') as string),
})
</script>

<template>
  <div class="flex flex-col w-full h-screen bg-background">
    <div class="border-b border-border bg-card p-4 md:p-6">
      <div class="max-w-6xl mx-auto space-y-4">
        <div>
          <h1 class="text-2xl md:text-3xl font-bold tracking-tight">
            {{ t('search.title') }}
          </h1>
          <p class="text-sm text-muted-foreground mt-1">
            {{ t('search.description') }}
          </p>
        </div>
        <div class="mt-4">
          <TantivySearchFilter
            :fields="searchFields"
            :model-value="searchQuery"
            @search="handleSearch"
            @update:model-value="searchQuery = $event"
          />
        </div>
      </div>
    </div>
    <div class="flex-1 overflow-hidden">
      <div
        v-if="loading"
        class="flex items-center justify-center h-full"
      >
        <div class="text-center">
          <Icon
            class="h-8 w-8 animate-spin text-muted-foreground mx-auto mb-4"
            name="lucide:loader-circle"
          />
          <p class="text-muted-foreground font-medium">{{ t('search.searching') }}</p>
          <p class="text-xs text-muted-foreground mt-1">{{ searchQuery }}</p>
        </div>
      </div>
      <div
        v-else-if="results.length === 0 && searchQuery"
        class="flex items-center justify-center h-full"
      >
        <div class="text-center max-w-md mx-auto px-4">
          <Icon
            class="h-12 w-12 text-muted-foreground mx-auto mb-4"
            name="lucide:inbox-x"
          />
          <h3 class="text-lg font-semibold">{{ t('search.noResults') }}</h3>
          <p class="text-muted-foreground text-sm mt-2">
            {{ t('search.noResultsDescription') }}
          </p>
          <p class="text-xs text-muted-foreground mt-4">
            {{ t('search.query') }}: <code class="bg-muted px-2 py-1 rounded">{{ searchQuery }}</code>
          </p>
          <button
            class="mt-4 text-sm text-blue-600 dark:text-blue-400 hover:underline"
            @click="searchQuery = undefined"
          >
            {{ t('search.clearSearch') }}
          </button>
        </div>
      </div>
      <ScrollArea
        v-else-if="results.length > 0"
        class="h-full"
      >
        <div class="max-w-6xl mx-auto">
          <div class="p-4 md:p-6 border-b border-border bg-card/50 sticky top-0 z-10">
            <p class="text-sm text-muted-foreground">
              <strong>{{ totalResults }}</strong>
              {{ totalResults === 1 ? t('search.oneResult') : t('search.multipleResults') }}
            </p>
          </div>
          <div class="divide-y divide-border">
            <EmailListItemComponent
              v-for="email in results"
              :key="email.id"
              :is-selected="selectedEmailIdFromRoute === email.id"
              class="border-b border-border transition-colors hover:bg-muted/50"
              v-bind="email"
              @click="selectEmail(email)"
            />
          </div>
        </div>
      </ScrollArea>
      <div
        v-else
        class="flex items-center justify-center h-full"
      >
        <div class="text-center max-w-md mx-auto px-4">
          <Icon
            class="h-12 w-12 text-muted-foreground mx-auto mb-4"
            name="lucide:search"
          />
          <h3 class="text-lg font-semibold">{{ t('search.title') }}</h3>
          <p class="text-muted-foreground text-sm mt-2">
            {{ t('search.helpText') }}
          </p>
      </div>
    </div>
    <UnobstrusiveSheetContent
      v-if="selectedEmailIdFromRoute"
      @close="onSheetChange(false)"
    >
      <ScrollArea class="h-full">
        <div class="flex h-full flex-col">
          <EmailViewer
            :key="selectedEmailIdFromRoute"
            :email-id="selectedEmailIdFromRoute"
          />
        </div>
      </ScrollArea>
    </UnobstrusiveSheetContent>
  </div>
</div></template>