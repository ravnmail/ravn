<script lang="ts" setup>
import { useAISearch } from '~/composables/useAISearch'

interface Props {
  modelValue?: string
  isLoading?: boolean
}

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'search', query: string): void
}>()

const { t } = useI18n()
const { generateSearchQuery, generating, error, resetError } = useAISearch()

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  isLoading: false,
})

const naturalLanguageInput = ref<string>('')
const queryInput = ref<string>(props.modelValue)

// Generate query from natural language and search
const handleNaturalLanguageSearch = async (e: Event) => {
  e.preventDefault()
  if (!naturalLanguageInput.value.trim() || generating.value) return

  const result = await generateSearchQuery(naturalLanguageInput.value)
  if (result) {
    queryInput.value = result
    emit('update:modelValue', result)
    emit('search', result)
    // Clear natural language input after successful search
    naturalLanguageInput.value = ''
    resetError()
  }
}

// Direct query input search
const handleQueryInputSearch = (e: Event) => {
  e.preventDefault()
  if (!queryInput.value.trim()) return

  emit('update:modelValue', queryInput.value)
  emit('search', queryInput.value)
  resetError()
}

const handleQueryInputChange = (e: Event) => {
  const value = (e.target as HTMLInputElement).value
  queryInput.value = value
  emit('update:modelValue', value)
  resetError()
}

watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal !== undefined && newVal !== queryInput.value) {
      queryInput.value = newVal
    }
  }
)
</script>

<template>
  <div class="w-full space-y-3">
    <form
      class="space-y-2"
      @submit="handleNaturalLanguageSearch"
    >
      <div
        class="relative flex items-center gap-2 px-4 py-3 bg-background border border-input rounded-lg focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2 transition-all"
      >
        <Icon
          class="text-ai flex-shrink-0"
          name="ravn:raven"
        />
        <input
          :disabled="generating || props.isLoading"
          :placeholder="t('search.ai.placeholder')"
          :value="naturalLanguageInput"
          autofocus
          class="flex-1 bg-transparent border-none outline-none text-sm placeholder-muted-foreground"
          type="text"
          @input="(e) => { naturalLanguageInput = (e.target as HTMLInputElement).value; resetError() }"
          @keydown.enter="handleNaturalLanguageSearch"
        >
        <button
          v-if="naturalLanguageInput && !generating && !props.isLoading"
          class="h-6 w-6 p-0.5 rounded hover:bg-muted transition-colors flex-shrink-0 text-muted-foreground hover:text-foreground"
          type="button"
          @click="naturalLanguageInput = ''"
        >
          <Icon
            class="w-4 h-4"
            name="lucide:x"
          />
        </button>
        <Icon
          v-if="generating || props.isLoading"
          class="w-4 h-4 animate-spin text-ai flex-shrink-0"
          name="lucide:loader-circle"
        />
        <button
          v-else-if="naturalLanguageInput.trim()"
          class="h-6 w-6 p-0.5 rounded hover:bg-muted transition-colors flex-shrink-0 text-muted-foreground hover:text-foreground"
          title="Search"
          type="submit"
        >
          <Icon
            class="w-4 h-4"
            name="lucide:arrow-right"
          />
        </button>
      </div>

      <!-- Error Message -->
      <div
        v-if="error"
        class="text-xs px-3 py-2 bg-red-50 dark:bg-red-950/20 text-red-600 dark:text-red-400 rounded"
      >
        {{ error }}
      </div>
    </form>

    <!-- Generated Query Field -->
    <Transition name="slide-fade">
      <form
        v-if="queryInput"
        class="space-y-1.5"
        @submit="handleQueryInputSearch"
      >
        <div
          class="relative flex items-center gap-2 px-4 py-3 bg-ai-background rounded-lg"
        >
          <input
            :disabled="props.isLoading"
            :value="queryInput"
            class="flex-1 bg-transparent border-none outline-none text-ai-foreground placeholder-muted-foreground"
            type="text"
            @input="handleQueryInputChange"
            @keydown.enter="handleQueryInputSearch"
          >
          <button
            v-if="queryInput && !props.isLoading"
            class="h-6 w-6 p-0.5 rounded text-ai"
            type="button"
            @click="queryInput = ''; emit('update:modelValue', '')"
          >
            <Icon
              name="lucide:x"
            />
          </button>
          <button
            class="h-6 w-6 p-0.5 rounded text-ai"
            title="Search"
            type="submit"
          >
            <Icon
              name="lucide:search"
            />
          </button>
        </div>
      </form>
    </Transition>
  </div>
</template>

<style scoped>
.slide-fade-enter-active {
  transition: all 0.3s ease;
}

.slide-fade-leave-active {
  transition: all 0.2s ease;
}

.slide-fade-enter-from {
  transform: translateY(-10px);
  opacity: 0;
}

.slide-fade-leave-to {
  transform: translateY(-10px);
  opacity: 0;
}
</style>
