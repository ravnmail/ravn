<script lang="ts" setup>

interface SearchField {
  type: 'text' | 'boolean' | 'datetime'
  label: string
  description?: string
}

interface SearchFields {
  [key: string]: SearchField
}

interface Token {
  type: 'operator' | 'field' | 'string' | 'date_range' | 'boolean' | 'group' | 'keyword'
  value: string
}

const props = defineProps<{
  fields: SearchFields
  modelValue?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue' | 'search', value: string): void
}>()

const { t } = useI18n()

// === STATE ===
const inputRef = ref<HTMLInputElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const query = ref<string>(props.modelValue || '')
const editingIndex = ref<number | null>(null)
const editingValue = ref<string>('')

// === PARSING ===
/**
 * Parse query string into tokens
 * This is the core parsing logic - keep it simple and correct
 */
const parseTokens = (q: string): Token[] => {
  if (!q.trim()) return []

  const result: Token[] = []
  let i = 0

  while (i < q.length) {
    const ch = q[i]

    // Skip whitespace
    if (/\s/.test(ch)) {
      i++
      continue
    }

    // Quoted string: "text" or "text with spaces"
    if (ch === '"') {
      let end = i + 1
      while (end < q.length && q[end] !== '"') {
        if (q[end] === '\\') end++ // Handle escaped quotes
        end++
      }
      if (end < q.length) end++ // Include closing quote
      result.push({
        type: 'string',
        value: q.slice(i, end),
      })
      i = end
      continue
    }

    // Date range: [2023-01-01 TO 2023-12-31]
    if (ch === '[') {
      let end = i + 1
      while (end < q.length && q[end] !== ']') end++
      if (end < q.length) end++ // Include closing bracket
      result.push({
        type: 'date_range',
        value: q.slice(i, end),
      })
      i = end
      continue
    }

    // Field:value pattern: from:email, subject:text, etc
    // Match: word characters, colon, then value (word chars, dashes, dots, @, *, ~, ?, or quoted/bracketed)
    const fieldPattern = /^(\w+):([\w\-.@*~?]+|"[^"]*"|\[[^\]]*\]|true|false)/
    const fieldMatch = q.slice(i).match(fieldPattern)
    if (fieldMatch) {
      const matched = fieldMatch[0]
      result.push({
        type: 'field',
        value: matched,
      })
      i += matched.length
      continue
    }

    // Boolean operators: AND, OR, NOT
    const opPattern = /^(AND|OR|NOT)(?=\s|$)/i
    const opMatch = q.slice(i).match(opPattern)
    if (opMatch) {
      result.push({
        type: 'operator',
        value: opMatch[1].toUpperCase(),
      })
      i += opMatch[1].length
      continue
    }

    // Parentheses
    if (ch === '(' || ch === ')') {
      result.push({
        type: 'group',
        value: ch,
      })
      i++
      continue
    }

    // Regular keyword/word
    const wordPattern = /^[^\s():\[\]"]+/
    const wordMatch = q.slice(i).match(wordPattern)
    if (wordMatch) {
      result.push({
        type: 'keyword',
        value: wordMatch[0],
      })
      i += wordMatch[0].length
      continue
    }

    // Fallback - skip character
    i++
  }

  return result
}

// === UI RENDERING ===
const getTokenClass = (token: Token): string => {
  const base = 'px-2 py-1 rounded text-xs font-mono font-semibold transition-colors border inline-flex items-center gap-1 hover:shadow-sm'

  const classes: Record<string, string> = {
    operator: `${base} bg-blue-100 text-blue-700 border-blue-300 dark:bg-blue-900/30 dark:text-blue-200 dark:border-blue-700`,
    field: `${base} bg-purple-100 text-purple-700 border-purple-300 dark:bg-purple-900/30 dark:text-purple-200 dark:border-purple-700`,
    string: `${base} bg-green-100 text-green-700 border-green-300 dark:bg-green-900/30 dark:text-green-200 dark:border-green-700`,
    date_range: `${base} bg-orange-100 text-orange-700 border-orange-300 dark:bg-orange-900/30 dark:text-orange-200 dark:border-orange-700`,
    boolean: `${base} bg-pink-100 text-pink-700 border-pink-300 dark:bg-pink-900/30 dark:text-pink-200 dark:border-pink-700`,
    group: `${base} bg-slate-200 text-slate-700 border-slate-400 dark:bg-slate-700/30 dark:text-slate-300 dark:border-slate-600`,
    keyword: `${base} bg-slate-100 text-slate-700 border-slate-300 dark:bg-slate-800/30 dark:text-slate-300 dark:border-slate-700`,
  }

  return classes[token.type] || base
}

// === ACTIONS ===
const removeToken = (index: number) => {
  const tokens = parseTokens(query.value)
  tokens.splice(index, 1)
  query.value = tokens.map(t => t.value).join(' ').trim()
}

const startEdit = (index: number) => {
  const tokens = parseTokens(query.value)
  editingIndex.value = index
  editingValue.value = tokens[index].value
}

const saveEdit = () => {
  if (editingIndex.value === null) return

  const tokens = parseTokens(query.value)
  if (editingValue.value.trim()) {
    tokens[editingIndex.value].value = editingValue.value
    query.value = tokens.map(t => t.value).join(' ').trim()
  }

  editingIndex.value = null
  editingValue.value = ''
}

const clearAll = () => {
  query.value = ''
}

// === KEYBOARD HANDLING ===
const handleKeyDown = (e: KeyboardEvent) => {
  // Enter: submit search
  if (e.key === 'Enter') {
    e.preventDefault()
    emit('search', query.value)
  }

  // Backspace at start with no tokens: delete last token
  if (
    e.key === 'Backspace' &&
    inputRef.value?.selectionStart === 0 &&
    inputRef.value?.selectionEnd === 0 &&
    inputRef.value?.value === ''
  ) {
    e.preventDefault()
    const tokens = parseTokens(query.value)
    if (tokens.length > 0) {
      tokens.pop()
      query.value = tokens.map(t => t.value).join(' ').trim()
    }
  }

  // Escape: close editor
  if (e.key === 'Escape' && editingIndex.value !== null) {
    editingIndex.value = null
  }
}

// === WATCHERS ===
watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal !== undefined && newVal !== query.value) {
      query.value = newVal
    }
  }
)

watch(query, (newVal) => {
  emit('update:modelValue', newVal)
})

// === LIFECYCLE ===
const handleClickOutside = (e: MouseEvent) => {
  if (containerRef.value && !containerRef.value.contains(e.target as Node)) {
    editingIndex.value = null
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside)
})
</script>

<template>
  <div
    ref="containerRef"
    class="w-full space-y-3"
  >
    <!-- Main Input Container -->
    <div
      class="flex flex-wrap gap-2 p-4 bg-background border border-input rounded-lg min-h-14 focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2 transition-all"
    >
      <!-- Display Tokens -->
      <div
        v-for="(token, index) in parseTokens(query)"
        :key="`token-${index}-${token.value}`"
        class="group relative"
      >
        <button
          :class="getTokenClass(token)"
          type="button"
          @click="startEdit(index)"
        >
          {{ token.value }}
        </button>

        <!-- Remove Button -->
        <button
          class="absolute -top-1.5 -right-1.5 opacity-0 group-hover:opacity-100 w-4 h-4 bg-red-500 hover:bg-red-600 text-white rounded-full flex items-center justify-center text-xs transition-opacity"
          type="button"
          @click.stop="removeToken(index)"
        >
          ×
        </button>
      </div>

      <!-- Input Field (Free text) -->
      <input
        ref="inputRef"
        :placeholder="t('labels.search.enterQuery')"
        :value="query"
        class="flex-1 min-w-48 bg-transparent border-none outline-none text-sm placeholder-muted-foreground font-mono"
        type="text"
        @input="(e) => query = (e.target as HTMLInputElement).value"
        @keydown="handleKeyDown"
      >

      <!-- Clear Button -->
      <button
        v-if="query"
        class="ml-auto h-6 w-6 p-0.5 rounded hover:bg-muted transition-colors flex-shrink-0 text-muted-foreground hover:text-foreground"
        type="button"
        @click="clearAll"
      >
        <svg
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
        >
          <line
            x1="18"
            x2="6"
            y1="6"
            y2="18"
          />
          <line
            x1="6"
            x2="18"
            y1="6"
            y2="18"
          />
        </svg>
      </button>
    </div>

    <!-- Token Editor Modal -->
    <div
      v-if="editingIndex !== null"
      class="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4"
      @click.self="editingIndex = null"
    >
      <div class="bg-background border border-input rounded-lg shadow-xl p-6 max-w-md w-full space-y-4">
        <h3 class="text-lg font-semibold">{{ t('labels.search.editToken') }}</h3>

        <div class="space-y-2">
          <label class="text-sm font-medium">{{ t('labels.search.value') }}</label>
          <input
            v-model="editingValue"
            autofocus
            class="w-full px-3 py-2 border border-input rounded-md bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            type="text"
            @keydown.enter="saveEdit"
            @keydown.escape="editingIndex = null"
          >
        </div>

        <div class="flex gap-2 justify-end pt-2">
          <button
            class="px-4 py-2 border border-input rounded-md hover:bg-muted transition-colors"
            type="button"
            @click="editingIndex = null"
          >
            {{ t('labels.common.cancel') }}
          </button>
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors"
            type="button"
            @click="saveEdit"
          >
            {{ t('labels.common.save') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Quick Reference -->
    <div class="text-xs text-muted-foreground space-y-1 px-3 py-2 bg-muted/30 rounded">
      <p class="font-semibold">{{ t('labels.search.examples') }}:</p>
      <ul class="space-y-0.5 list-disc list-inside">
        <li><code class="bg-background px-1 rounded">from:john@example.com AND subject:urgent</code></li>
        <li><code class="bg-background px-1 rounded">"exact phrase"</code></li>
        <li><code class="bg-background px-1 rounded">is_read:true</code></li>
      </ul>
    </div>
  </div>
</template>