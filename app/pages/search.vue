<script lang="ts" setup>
import type { EmailListItem } from '~/types/email'
import { ScrollArea } from '~/components/ui/scroll-area'
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import EmailViewer from '~/components/Ravn/EmailViewer.vue'
import EmailListItemComponent from '~/components/Ravn/EmailListItem.vue'
import TantivySearchFilter from '~/components/Ravn/TantivySearchFilter.vue'
import AISearchInput from '~/components/Ravn/AISearchInput.vue'
import { useTantivySearch, type SearchFields } from '~/composables/useTantivySearch'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import { ToggleGroup, ToggleGroupItem } from '~/components/ui/toggle-group'

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

const mode = computed({
  get() {
    return (route.query.mode as string) || 'search'
  },
  set(value: string) {
    const query = { ...route.query, mode: value }
    router.replace({ query })
  },
})

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

const performSearch = async () => {
  if (searchQuery.value) {
    await search({ query: searchQuery.value, limit: 200 })
  }
}

watch(searchQuery, (newQuery) => {
  if (newQuery && newQuery.trim() !== '') {
    performSearch()
  }
}, { immediate: true })

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
    <div class="border-b border-border bg-background p-2">
      <div class="max-w-7xl mx-auto space-y-2">
        <div class="flex items-center justify-end">
          <ToggleGroup
            v-model="mode"
            class="relative z-10"
            type="single"
          >
            <ToggleGroupItem
              type="button"
              value="search"
            >
              <span>{{ t('search.title') }}</span>
            </ToggleGroupItem>
            <ToggleGroupItem
              type="button"
              value="ai"
            >
              <Icon name="ravn:raven"/>
              <span>{{ t('search.ai.toggle') }}</span>
            </ToggleGroupItem>
          </ToggleGroup>
        </div>
        <AISearchInput
          v-if="mode === 'ai'"
          :model-value="searchQuery"
          @search="handleSearch"
          @update:model-value="searchQuery = $event"
        />
        <TantivySearchFilter
          v-if="mode === 'search'"
          :fields="searchFields"
          :model-value="searchQuery"
          @search="handleSearch"
          @update:model-value="searchQuery = $event"
        />
      </div>
    </div>
    <div class="flex-1 bg-surface overflow-hidden">
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
      <EmptyState
        v-else-if="results.length === 0 && searchQuery"
        :description="t('search.noResultsDescription')"
        :title="t('search.noResults')"
        class="h-full"
        icon="🔎"
      >
        <button
          class="mt-4 text-sm text-blue-600 dark:text-blue-400 hover:underline"
          @click="searchQuery = undefined"
        >
          {{ t('search.clearSearch') }}
        </button>
      </EmptyState>
      <ScrollArea
        v-else-if="results.length > 0"
        class="h-full"
      >
        <div class="max-w-7xl mx-auto space-y-0.5">
          <div class="p-2 sticky top-0 z-10 bg-surface">
            <p class="text-sm">
              <strong>{{ totalResults }}</strong>
              {{ totalResults === 1 ? t('search.oneResult') : t('search.multipleResults') }}
            </p>
          </div>
          <EmailListItemComponent
            v-for="email in results"
            :key="email.id"
            :is-selected="selectedEmailIdFromRoute === email.id"
            class="transition-colors hover:bg-muted/50"
            v-bind="email"
            @click="selectEmail(email)"
          />
        </div>
      </ScrollArea>
      <EmptyState
        v-else
        :description="t('search.helpText')"
        :title="t('search.title')"
        class="h-full"
        icon="🔎"
      />
      <UnobstrusiveSheetContent
        v-if="selectedEmailIdFromRoute"
        @close="onSheetChange(false)"
      >
        <ScrollArea class="w-full">
          <div class="flex h-full flex-col">
            <EmailViewer
              :key="selectedEmailIdFromRoute"
              :email-id="selectedEmailIdFromRoute"
            />
          </div>
        </ScrollArea>
      </UnobstrusiveSheetContent>
    </div>
  </div>
</template>