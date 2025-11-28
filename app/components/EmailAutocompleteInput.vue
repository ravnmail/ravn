<script lang="ts" setup>
import { onClickOutside } from '@vueuse/core'
import type { ContactSummary } from '~/types/contact'
import {
  TagsInput,
  TagsInputItem,
  TagsInputItemText,
  TagsInputItemDelete,
  TagsInputInput
} from '~/components/ui/tags-input'

const { t } = useI18n()

interface Props {
  modelValue: string[]
  accountId: string
  placeholder?: string
  delimiter?: RegExp
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Enter email...',
  delimiter: () => /[\s,;]+/
})

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const searchQuery = ref('')

const { useSearchContacts, useGetTopContacts } = useContacts()
const { data: topAccounts } = useGetTopContacts({
  account_id: props.accountId,
  limit: 6,
})
const { data: results } = useSearchContacts(searchQuery, {
  account_id: props.accountId,
  limit: 6
})

const isOpen = ref(false)
const containerRef = ref<HTMLElement>()
const selectedIndex = ref(-1)
let inputListener: ((e: Event) => void) | null = null

const handleFocus = () => {
  isOpen.value = true
}

const selectContact = (contact: ContactSummary) => {
  const email = contact.email

  if (!props.modelValue.includes(email)) {
    emit('update:modelValue', [...props.modelValue, email])
  }

  const inputElement = containerRef.value?.querySelector('input')
  if (inputElement) {
    inputElement.value = ''
    inputElement.focus()
  }
  searchQuery.value = ''
  isOpen.value = false
  selectedIndex.value = -1
}

const handleKeyDown = (event: KeyboardEvent) => {
  if (!isOpen.value || suggestions.value.length === 0) return
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, suggestions.value.length - 1)
      break
    case 'ArrowUp':
      event.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, -1)
      break
    case 'Enter':
      if (selectedIndex.value >= 0 && selectedIndex.value < suggestions.value.length) {
        event.preventDefault()
        selectContact(suggestions.value[selectedIndex.value])
        searchQuery.value = ''
        const inputElement = containerRef.value?.querySelector('input')
        if (inputElement) {
          inputElement.value = ''
        }
      }
      break
    case 'Escape':
      event.preventDefault()
      event.stopPropagation()
      isOpen.value = false
      selectedIndex.value = -1
      searchQuery.value = ''
      break
  }
}

const handleModelValueChange = (value: string[]) => {
  emit('update:modelValue', value)

  if (value.length > props.modelValue.length) {
    searchQuery.value = ''
  }
}

onClickOutside(containerRef, () => {
  isOpen.value = false
  selectedIndex.value = -1
})

const suggestions = computed<ContactSummary[]>(() => {
  if (searchQuery.value.trim()) {
    return results.value || []
  } else {
    return topAccounts.value || []
  }
})

watch(suggestions, () => {
  selectedIndex.value = -1
})

onMounted(() => {
  setTimeout(() => {
    if (containerRef.value) {
      const inputElement = containerRef.value.querySelector('input')
      if (inputElement) {
        inputListener = (e: Event) => {
          const target = e.target as HTMLInputElement
          const value = target.value
          searchQuery.value = value

          if (!isOpen.value && value) {
            isOpen.value = true
          }
        }

        inputElement.addEventListener('input', inputListener)
      }
    }
  }, 100)
})

onUnmounted(() => {
  if (containerRef.value && inputListener) {
    const inputElement = containerRef.value.querySelector('input')
    if (inputElement) {
      inputElement.removeEventListener('input', inputListener)
    }
  }
})
</script>

<template>
  <div
    ref="containerRef"
    class="relative flex-1"
  >
    <TagsInput
      :delimiter="delimiter"
      :model-value="modelValue"
      class="flex-1"
      @update:model-value="(v) => handleModelValueChange(v as string[])"
    >
      <TagsInputItem
        v-for="email in modelValue"
        :key="email"
        :value="email"
      >
        <RavnAvatar
          :email="email"
          class="ml-1"
          size="xs"
        />
        <TagsInputItemText/>
        <TagsInputItemDelete/>
      </TagsInputItem>
      <TagsInputInput
        :placeholder="placeholder"
        @click="handleFocus"
        @focus="handleFocus"
        @keydown="handleKeyDown"
      />
    </TagsInput>
    <div
      v-show="isOpen"
      class="absolute left-0 right-0 top-full z-50 mt-1 bg-popover text-popover-foreground border border-border rounded-md shadow-lg max-h-96 overflow-y-auto"
    >
      <div
        v-if="suggestions.length === 0"
        class="px-3 py-2 text-sm text-muted-foreground"
      >
        {{ searchQuery ? t('components.emailAutocomplete.noResults') : t('components.emailAutocomplete.noContacts') }}
      </div>
      <div
        v-for="(contact, index) in suggestions"
        :key="contact.id"
        :class="{ 'bg-accent text-accent-foreground': index === selectedIndex }"
        :data-index="index"
        class="flex items-center gap-2 px-3 py-2 hover:bg-accent hover:text-accent-foreground transition-colors"
        @click="selectContact(contact)"
        @mouseenter="selectedIndex = index"
      >
        <RavnAvatar
          :email="contact.email"
          class="flex-shrink-0"
        />

        <div class="flex-1 min-w-0">
          <div class="font-semibold text-sm truncate">
            {{ contact.display_name || contact.email }}
          </div>
          <div
            v-if="contact.display_name"
            class="text-xs truncate opacity-70"
          >
            {{ contact.email }}
          </div>
        </div>
        <div
          v-if="contact.send_count > 0"
          class="flex-shrink-0 text-xs opacity-70"
        >
          {{ contact.send_count }}
        </div>
      </div>
    </div>
  </div>
</template>
