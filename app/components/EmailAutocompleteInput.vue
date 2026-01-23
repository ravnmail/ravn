<script lang="ts" setup>
import { draggable, dropTargetForElements } from '@atlaskit/pragmatic-drag-and-drop/element/adapter'
import type { CleanupFn } from '@atlaskit/pragmatic-drag-and-drop/types'
import { onClickOutside } from '@vueuse/core'
import {
  TagsInput,
  TagsInputInput,
  TagsInputItem,
  TagsInputItemDelete,
} from '~/components/ui/tags-input'
import { formatEmailAddress, parseEmailAddress } from '~/lib/utils/email'
import type { ContactSummary } from '~/types/contact'
import type { EmailAddress } from '~/types/email'

const { t } = useI18n()

interface Props {
  modelValue: EmailAddress[]
  placeholder?: string
  delimiter?: RegExp
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Enter email...',
  delimiter: () => /[\s,;]+/,
})

const emit = defineEmits<{
  'update:modelValue': [value: EmailAddress[]]
}>()

// Drag and drop state
interface EmailTokenDragData {
  type: 'email-token'
  email: EmailAddress
  sourceInstanceId: string
  emailString: string
  isCopy: boolean
  [key: string]: unknown
}

const instanceId = `email-autocomplete-${Math.random().toString(36).substr(2, 9)}`
const dragCleanups = ref<Map<string, CleanupFn>>(new Map())
const isDropTarget = ref(false)
const canAcceptDrop = ref(false)
const isDragCopy = ref(false)
let currentDragAltKey = false

const searchQuery = ref('')

const { useSearchContacts, useGetTopContacts } = useContacts()
const { data: topAccounts } = useGetTopContacts({
  limit: 12,
})
const { data: results } = useSearchContacts(searchQuery, {
  limit: 12,
})

const isOpen = ref(false)
const containerRef = ref<HTMLElement>()
const selectedIndex = ref(-1)
let inputListener: ((e: Event) => void) | null = null
let dropTargetCleanup: CleanupFn | null = null

const handleFocus = () => {
  isOpen.value = true
}

const selectContact = (contact: ContactSummary) => {
  emit('update:modelValue', [
    ...props.modelValue,
    { name: contact.display_name, address: contact.email },
  ])

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

const handleModelValueChange = (value: Array<EmailAddress | string>) => {
  emit(
    'update:modelValue',
    value.map((v: EmailAddress | string): EmailAddress => {
      if (typeof v === 'string') {
        return parseEmailAddress(v)
      }
      return v
    })
  )

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

const setupDraggableToken = (element: HTMLElement, email: EmailAddress) => {
  const key = email.address

  const existingCleanup = dragCleanups.value.get(key)
  if (existingCleanup) {
    existingCleanup()
  }

  const emailString = formatEmailAddress(email)
  const handleMouseDown = (e: MouseEvent) => {
    currentDragAltKey = e.altKey
  }

  element.addEventListener('mousedown', handleMouseDown)

  const cleanup = draggable({
    element,
    getInitialData: () => ({
      type: 'email-token',
      email,
      sourceInstanceId: instanceId,
      emailString,
      isCopy: currentDragAltKey,
    }),
    onDragStart: () => {
      element.style.opacity = '0.5'
      isDragCopy.value = currentDragAltKey
    },
    onDrop: () => {
      element.style.opacity = '1'
      isDragCopy.value = false
      currentDragAltKey = false
    },
  })

  // Return original cleanup wrapped to also remove mousedown listener
  const originalCleanup = cleanup
  const wrappedCleanup = () => {
    element.removeEventListener('mousedown', handleMouseDown)
    originalCleanup()
  }

  dragCleanups.value.set(key, wrappedCleanup)
}

const setupDropTarget = (element: HTMLElement) => {
  if (dropTargetCleanup) {
    dropTargetCleanup()
  }

  dropTargetCleanup = dropTargetForElements({
    element,
    canDrop: (args) => {
      const data = args.source.data as unknown as EmailTokenDragData
      return data.type === 'email-token' && data.sourceInstanceId !== instanceId
    },
    onDragEnter: (args) => {
      const data = args.source.data as unknown as EmailTokenDragData
      isDropTarget.value = true
      canAcceptDrop.value = data.type === 'email-token' && data.sourceInstanceId !== instanceId
    },
    onDragLeave: () => {
      isDropTarget.value = false
      canAcceptDrop.value = false
    },
    onDrop: (args) => {
      const data = args.source.data as unknown as EmailTokenDragData
      isDropTarget.value = false
      canAcceptDrop.value = false

      if (data.type === 'email-token' && data.sourceInstanceId !== instanceId) {
        const exists = props.modelValue.some((e) => e.address === data.email.address)
        if (!exists) {
          emit('update:modelValue', [...props.modelValue, data.email])
        }

        // Check if Alt key was pressed (copy mode)
        const isCopyMode = data.isCopy || isDragCopy.value

        window.dispatchEvent(
          new CustomEvent('email-token-dropped', {
            detail: {
              email: data.email,
              sourceInstanceId: data.sourceInstanceId,
              targetInstanceId: instanceId,
              isCopyMode,
            },
          })
        )
      }
    },
  })
}

const handleEmailTokenDropped = (event: CustomEvent) => {
  const { email, sourceInstanceId, targetInstanceId, isCopyMode } = event.detail
  if (sourceInstanceId === instanceId && !isCopyMode && targetInstanceId) {
    emit(
      'update:modelValue',
      props.modelValue.filter((e) => e.address !== email.address)
    )
  }
}

watch(containerRef, (newContainer) => {
  if (newContainer) {
    setupDropTarget(newContainer)
  }
})

onMounted(() => {
  window.addEventListener('email-token-dropped', handleEmailTokenDropped as EventListener)
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
  window.removeEventListener('email-token-dropped', handleEmailTokenDropped as EventListener)

  if (containerRef.value && inputListener) {
    const inputElement = containerRef.value.querySelector('input')
    if (inputElement) {
      inputElement.removeEventListener('input', inputListener)
    }
  }

  dragCleanups.value.forEach((cleanup) => {
    cleanup()
  })
  dragCleanups.value.clear()

  if (dropTargetCleanup) {
    dropTargetCleanup()
  }
})
</script>

<template>
  <div
    ref="containerRef"
    class="relative flex-1 transition-all"
    :class="{
      'ring-1 ring-selection ring-offset-2': isDropTarget && canAcceptDrop,
    }"
    c
  >
    <TagsInput
      :delimiter="delimiter"
      :model-value="modelValue"
      class="flex-1"
      @update:model-value="(v) => handleModelValueChange(v as EmailAddress[])"
    >
      <TagsInputItem
        v-for="email in modelValue"
        :key="email.address"
        :ref="
          (el: any) => {
            if (el && el.$el) {
              setupDraggableToken(el.$el as HTMLElement, email)
            }
          }
        "
        :value="email"
      >
        <RavnAvatar
          :email="email.address"
          :name="email.name"
          class="pointer-events-none ml-1"
          size="xs"
        />
        <span class="px-2 py-0.5 text-sm">{{
          email.name ? `${email.name} <${email.address}>` : email.address
        }}</span>
        <TagsInputItemDelete />
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
      class="absolute top-full right-0 left-0 z-50 mt-1 max-h-96 overflow-y-auto rounded-md border border-popover-border bg-popover p-1 text-popover-foreground shadow-lg"
    >
      <div
        v-if="suggestions.length === 0"
        class="text-muted-foreground px-3 py-2 text-sm"
      >
        {{
          searchQuery
            ? t('components.emailAutocomplete.noResults')
            : t('components.emailAutocomplete.noContacts')
        }}
      </div>
      <div
        v-for="(contact, index) in suggestions"
        :key="contact.id"
        :class="{ 'bg-selection text-selection-foreground': index === selectedIndex }"
        :data-index="index"
        class="flex items-center gap-2 rounded p-2 transition-colors hover:bg-selection hover:text-selection-foreground"
        @click="selectContact(contact)"
        @mouseenter="selectedIndex = index"
      >
        <RavnAvatar
          :email="contact.email"
          class="shrink-0"
        />

        <div class="min-w-0 flex-1">
          <div class="truncate text-sm font-semibold">
            {{ contact.display_name || contact.email }}
          </div>
          <div
            v-if="contact.display_name"
            class="truncate text-xs opacity-70"
          >
            {{ contact.email }}
          </div>
        </div>
        <div class="flex-shrink-0 text-xs opacity-60">
          {{ contact.send_count }}
        </div>
      </div>
    </div>
  </div>
</template>
