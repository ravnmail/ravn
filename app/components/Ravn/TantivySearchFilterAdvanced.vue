<script lang="ts" setup>
import type { Dayjs } from 'dayjs'
import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'
import customParseFormat from 'dayjs/plugin/customParseFormat'

dayjs.extend(utc)
dayjs.extend(customParseFormat)

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
  fieldName?: string
  fieldType?: SearchField['type']
}

interface EditingState {
  index: number
  token: Token
  originalValue: string
  isEditing: boolean
}

const props = defineProps<{
  fields: SearchFields
  modelValue?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue' | 'search', value: string): void
}>()

const { t } = useI18n()

const inputRef = ref<HTMLInputElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const query = ref<string>(props.modelValue || '')
const editing = ref<EditingState | null>(null)
const focusedTokenIndex = ref<number | null>(null)
const showFieldSuggestions = ref(false)
const selectedSuggestionIndex = ref(0)

/**
 * Parse query string into tokens with full metadata
 */
const parseTokens = (q: string): Token[] => {
  if (!q.trim()) return []

  const result: Token[] = []
  let i = 0

  while (i < q.length) {
    const ch = q[i]

    if (/\s/.test(ch)) {
      i++
      continue
    }

    if (ch === '"') {
      let end = i + 1
      while (end < q.length && q[end] !== '"') {
        if (q[end] === '\\') end++
        end++
      }
      if (end < q.length) end++
      result.push({
        type: 'string',
        value: q.slice(i, end),
      })
      i = end
      continue
    }

    if (ch === '[') {
      let end = i + 1
      while (end < q.length && q[end] !== ']') end++
      if (end < q.length) end++
      result.push({
        type: 'date_range',
        value: q.slice(i, end),
      })
      i = end
      continue
    }

    const fieldPattern = /^(\w+):([\w\-\.@*~?]+|"[^"]*"|\[[^\]]*\]|true|false)/
    const fieldMatch = q.slice(i).match(fieldPattern)
    if (fieldMatch) {
      const matched = fieldMatch[0]
      const fieldName = fieldMatch[1]
      const fieldDef = props.fields[fieldName.toLowerCase()]

      result.push({
        type: 'field',
        value: matched,
        fieldName: fieldName.toLowerCase(),
        fieldType: fieldDef?.type,
      })
      i += matched.length
      continue
    }

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

    if (ch === '(' || ch === ')') {
      result.push({
        type: 'group',
        value: ch,
      })
      i++
      continue
    }

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

    i++
  }

  return result
}

const tokens = computed(() => parseTokens(query.value))

const getTokenClass = (token: Token): string => {
  const base = 'relative px-2 py-1 rounded text-xs font-mono font-semibold transition-all border inline-flex items-center gap-1 group'

  const classes: Record<string, string> = {
    operator: `${base} bg-blue-100 text-blue-700 border-blue-300 dark:bg-blue-900/30 dark:text-blue-200 dark:border-blue-700 hover:shadow-md`,
    field: `${base} bg-purple-100 text-purple-700 border-purple-300 dark:bg-purple-900/30 dark:text-purple-200 dark:border-purple-700 hover:shadow-md`,
    string: `${base} bg-green-100 text-green-700 border-green-300 dark:bg-green-900/30 dark:text-green-200 dark:border-green-700 hover:shadow-md`,
    date_range: `${base} bg-orange-100 text-orange-700 border-orange-300 dark:bg-orange-900/30 dark:text-orange-200 dark:border-orange-700 hover:shadow-md`,
    boolean: `${base} bg-pink-100 text-pink-700 border-pink-300 dark:bg-pink-900/30 dark:text-pink-200 dark:border-pink-700 hover:shadow-md`,
    group: `${base} bg-slate-200 text-slate-700 border-slate-400 dark:bg-slate-700/30 dark:text-slate-300 dark:border-slate-600 hover:shadow-md`,
    keyword: `${base} bg-slate-100 text-slate-700 border-slate-300 dark:bg-slate-800/30 dark:text-slate-300 dark:border-slate-700 hover:shadow-md`,
  }

  return classes[token.type] || base
}

const getFieldValue = (fieldValue: string): { field: string; value: string } => {
  const colonIndex = fieldValue.indexOf(':')
  if (colonIndex === -1) return { field: '', value: '' }
  return {
    field: fieldValue.substring(0, colonIndex),
    value: fieldValue.substring(colonIndex + 1),
  }
}

const buildFieldToken = (fieldName: string, value: string): string => {
  const field = props.fields[fieldName]
  if (!field) return `${fieldName}:`

  switch (field.type) {
    case 'boolean':
      return `${fieldName}:${value === 'true' ? 'true' : 'false'}`
    case 'datetime':
      return `${fieldName}:${value}`
    case 'text':
    default:
      if (value.includes(' ')) {
        return `${fieldName}:"${value}"`
      }
      return `${fieldName}:${value}`
  }
}

const startEdit = (index: number) => {
  const token = tokens.value[index]
  if (token.type !== 'field') {
    focusedTokenIndex.value = index
    nextTick(() => {
      inputRef.value?.focus()
    })
    return
  }

  editing.value = {
    index,
    token: { ...token },
    originalValue: token.value,
    isEditing: true,
  }
  focusedTokenIndex.value = index
}

const saveEdit = () => {
  if (!editing.value) return

  const { index, token } = editing.value
  const newTokens = [...tokens.value]
  newTokens[index] = token
  query.value = newTokens.map(t => t.value).join(' ').trim()
  editing.value = null
  focusedTokenIndex.value = null
}

const cancelEdit = () => {
  editing.value = null
  focusedTokenIndex.value = null
}

const removeToken = (index: number) => {
  const newTokens = tokens.value.filter((_, i) => i !== index)
  query.value = newTokens.map(t => t.value).join(' ').trim()
  editing.value = null
  focusedTokenIndex.value = null
}

const setBooleanValue = (value: boolean) => {
  if (!editing.value || editing.value.token.type !== 'field') return
  const fieldName = editing.value.token.fieldName!
  editing.value.token.value = buildFieldToken(fieldName, value ? 'true' : 'false')
}

const parseNaturalDate = (input: string): Dayjs | null => {
  const now = dayjs()
  const lower = input.toLowerCase().trim()

  const patterns: Record<string, () => Dayjs> = {
    'today': () => now,
    'yesterday': () => now.subtract(1, 'day'),
    'tomorrow': () => now.add(1, 'day'),
    'last week': () => now.subtract(1, 'week'),
    'this week': () => now.startOf('week'),
    'last month': () => now.subtract(1, 'month'),
    'this month': () => now.startOf('month'),
    'last year': () => now.subtract(1, 'year'),
    'this year': () => now.startOf('year'),
  }

  if (patterns[lower]) {
    return patterns[lower]()
  }

  const formats = ['YYYY-MM-DD', 'DD.MM.YYYY', 'DD/MM/YYYY', 'MM/DD/YYYY']
  for (const format of formats) {
    const parsed = dayjs(input, format, true)
    if (parsed.isValid()) return parsed
  }

  return null
}

const setDateRange = (startStr: string, endStr: string) => {
  if (!editing.value || editing.value.token.type !== 'field') return

  const start = parseNaturalDate(startStr)
  const end = parseNaturalDate(endStr)

  if (!start || !end) return

  const startISO = start.utc().format('YYYY-MM-DDTHH:mm:ss[Z]')
  const endISO = end.utc().format('YYYY-MM-DDTHH:mm:ss[Z]')
  const fieldName = editing.value.token.fieldName!
  editing.value.token.value = `${fieldName}:[${startISO} TO ${endISO}]`
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (editing.value) {
    return
  }

  if (e.key === 'Enter') {
    e.preventDefault()
    showFieldSuggestions.value = false
    emit('search', query.value)
    return
  }

  if (e.key === 'Escape') {
    e.preventDefault()
    showFieldSuggestions.value = false
    return
  }

  if (e.key === 'Tab' && showFieldSuggestions.value) {
    e.preventDefault()
    const suggestions = Object.keys(props.fields)
    addField(suggestions[selectedSuggestionIndex.value])
    return
  }

  if (
    e.key === 'Backspace' &&
    inputRef.value?.selectionStart === 0 &&
    inputRef.value?.selectionEnd === 0 &&
    !inputRef.value?.value
  ) {
    e.preventDefault()
    const newTokens = tokens.value.slice(0, -1)
    query.value = newTokens.map(t => t.value).join(' ').trim()
  }
}

const addField = (fieldName: string) => {
  const newPart = `${fieldName}:`
  if (query.value && !query.value.endsWith(' ')) {
    query.value += ' ' + newPart
  } else {
    query.value += newPart
  }
  showFieldSuggestions.value = false
  nextTick(() => {
    inputRef.value?.focus()
  })
}

const fieldSuggestions = computed(() => {
  return Object.keys(props.fields).filter(name => {
    const token = tokens.value.find(t => t.fieldName === name && t.type === 'field')
    return !token
  })
})

const clearAll = () => {
  query.value = ''
  editing.value = null
  focusedTokenIndex.value = null
  showFieldSuggestions.value = false
}

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

const handleClickOutside = (e: MouseEvent) => {
  if (containerRef.value && !containerRef.value.contains(e.target as Node)) {
    editing.value = null
    focusedTokenIndex.value = null
    showFieldSuggestions.value = false
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
    <div class="space-y-2">
      <div
        class="flex flex-wrap gap-2 p-4 bg-background border border-input rounded-lg min-h-14 focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2 transition-all"
      >
        <div
          v-for="(token, idx) in tokens"
          :key="`token-${idx}-${token.value}`"
          class="relative"
        >
          <!-- Token Display/Edit -->
          <div
            v-if="!editing || editing.index !== idx"
            :class="getTokenClass(token)"
            :title="`Click to edit • Type: ${token.type}`"
            @click="startEdit(idx)"
          >
            <span class="truncate max-w-xs">{{ token.value }}</span>
            <button
              :title="t('labels.common.delete')"
              class="opacity-0 group-hover:opacity-100 ml-1 -mr-1 h-4 w-4 flex items-center justify-center rounded hover:bg-red-500/20 text-red-600 hover:text-red-700 transition-all"
              type="button"
              @click.stop="removeToken(idx)"
            >
              ×
            </button>
          </div>
          <div
            v-else-if="editing.index === idx && token.type === 'field'"
            :class="getTokenClass(token)"
            @click.stop
          >
            <div
              v-if="editing.token.fieldType === 'text'"
              class="flex items-center gap-1"
            >
              <input
                :value="getFieldValue(editing.token.value).value"
                autofocus
                class="bg-transparent border-b border-current outline-none w-32 text-xs"
                placeholder="value"
                type="text"
                @input="(e) => {
                  const val = (e.target as HTMLInputElement).value
                  const fieldName = editing.token.fieldName!
                  editing.token.value = buildFieldToken(fieldName, val)
                }"
                @keydown.enter="saveEdit"
                @keydown.escape="cancelEdit"
              >
            </div>

            <div
              v-else-if="editing.token.fieldType === 'boolean'"
              class="flex items-center gap-1"
            >
              <button
                :class="[
                  'px-1.5 py-0.5 rounded text-xs font-bold transition-all',
                  getFieldValue(editing.token.value).value === 'true'
                    ? 'bg-green-500 text-white'
                    : 'bg-slate-200 dark:bg-slate-700 hover:bg-green-500/30'
                ]"
                type="button"
                @click="setBooleanValue(true)"
              >
                T
              </button>
              <button
                :class="[
                  'px-1.5 py-0.5 rounded text-xs font-bold transition-all',
                  getFieldValue(editing.token.value).value === 'false'
                    ? 'bg-red-500 text-white'
                    : 'bg-slate-200 dark:bg-slate-700 hover:bg-red-500/30'
                ]"
                type="button"
                @click="setBooleanValue(false)"
              >
                F
              </button>
            </div>
            <div
              v-else-if="editing.token.fieldType === 'datetime'"
              class="flex items-center gap-1"
            >
              <input
                class="bg-transparent border-b border-current outline-none text-xs"
                type="date"
                @input="(e) => {
                  const endVal = getFieldValue(editing.token.value).value
                  const endMatch = endVal.match(/TO ([^\]]+)/)
                  const end = endMatch ? endMatch[1].trim().split('T')[0] : ''
                  setDateRange((e.target as HTMLInputElement).value, end)
                }"
                @keydown.enter="saveEdit"
                @keydown.escape="cancelEdit"
              >
              <span class="text-xs">→</span>
              <input
                class="bg-transparent border-b border-current outline-none text-xs"
                type="date"
                @input="(e) => {
                  const startVal = getFieldValue(editing.token.value).value
                  const startMatch = startVal.match(/\[([^\s]+)/)
                  const start = startMatch ? startMatch[1].split('T')[0] : ''
                  setDateRange(start, (e.target as HTMLInputElement).value)
                }"
                @keydown.enter="saveEdit"
                @keydown.escape="cancelEdit"
              >
            </div>
            <div
              v-else
              class="flex items-center gap-1"
            >
              <input
                :value="getFieldValue(editing.token.value).value"
                autofocus
                class="bg-transparent border-b border-current outline-none w-32 text-xs"
                placeholder="value"
                type="text"
                @input="(e) => {
                  const val = (e.target as HTMLInputElement).value
                  const fieldName = editing.token.fieldName!
                  editing.token.value = buildFieldToken(fieldName, val)
                }"
                @keydown.enter="saveEdit"
                @keydown.escape="cancelEdit"
              >
            </div>
            <div class="flex items-center gap-0.5 ml-auto">
              <button
                :title="t('labels.common.save')"
                class="px-1 hover:bg-white/20 rounded text-xs"
                type="button"
                @click="saveEdit"
              >
                ✓
              </button>
              <button
                :title="t('labels.common.cancel')"
                class="px-1 hover:bg-red-500/20 text-red-600 rounded text-xs"
                type="button"
                @click="cancelEdit"
              >
                ✕
              </button>
            </div>
          </div>
          <div
            v-else
            :class="getTokenClass(token)"
            @click.stop
          >
            <input
              :value="editing.token.value"
              autofocus
              class="bg-transparent border-b border-current outline-none w-32 text-xs"
              type="text"
              @input="(e) => editing.token.value = (e.target as HTMLInputElement).value"
              @keydown.enter="saveEdit"
              @keydown.escape="cancelEdit"
            >
            <div class="flex items-center gap-0.5 ml-auto">
              <button
                class="px-1 hover:bg-white/20 rounded text-xs"
                type="button"
                @click="saveEdit"
              >✓
              </button>
              <button
                class="px-1 hover:bg-red-500/20 text-red-600 rounded text-xs"
                type="button"
                @click="cancelEdit"
              >✕
              </button>
            </div>
          </div>
        </div>
        <input
          ref="inputRef"
          :placeholder="t('labels.search.enterQuery')"
          :value="query"
          class="flex-1 min-w-48 bg-transparent border-none outline-none text-sm placeholder-muted-foreground font-mono"
          type="text"
          @focus="() => {
            if (query.endsWith(':')) {
              showFieldSuggestions.value = true
            }
          }"
          @input="(e) => query = (e.target as HTMLInputElement).value"
          @keydown="handleKeyDown"
        >
        <button
          v-if="query"
          :title="t('labels.common.clear')"
          class="h-6 w-6 p-0.5 rounded hover:bg-muted transition-colors flex-shrink-0 text-muted-foreground hover:text-foreground"
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
      <div
        v-if="showFieldSuggestions && fieldSuggestions.length > 0"
        class="bg-background border border-input rounded-lg shadow-lg overflow-hidden max-w-sm"
      >
        <div class="p-2 space-y-1 max-h-48 overflow-y-auto">
          <button
            v-for="(field, idx) in fieldSuggestions"
            :key="field"
            :class="[
              'w-full text-left px-3 py-2 rounded text-sm transition-colors text-xs',
              idx === selectedSuggestionIndex
                ? 'bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100'
                : 'hover:bg-muted'
            ]"
            type="button"
            @click="addField(field)"
          >
            <div class="font-semibold">{{ field }}:</div>
            <div class="text-muted-foreground text-xs">{{ props.fields[field].label }}</div>
          </button>
        </div>
      </div>
    </div>
    <div class="text-xs text-muted-foreground space-y-1 px-3 py-2 bg-muted/30 rounded">
      <p class="font-semibold">{{ t('labels.search.examples') }}:</p>
      <ul class="space-y-0.5 list-disc list-inside">
        <li><code class="bg-background px-1 rounded">from:john@example.com AND subject:urgent</code></li>
        <li><code class="bg-background px-1 rounded">"exact phrase"</code></li>
        <li><code class="bg-background px-1 rounded">is_read:true</code></li>
        <li class="text-xs text-muted-foreground/70">Click any token to edit it • Press Escape to cancel • Enter to
          save
        </li>
      </ul>
    </div>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs">
      <div class="flex items-center gap-2 px-2 py-1 bg-muted/30 rounded">
        <div class="w-2 h-2 rounded-full bg-blue-600"/>
        <span>Operators</span>
      </div>
      <div class="flex items-center gap-2 px-2 py-1 bg-muted/30 rounded">
        <div class="w-2 h-2 rounded-full bg-purple-600"/>
        <span>Fields</span>
      </div>
      <div class="flex items-center gap-2 px-2 py-1 bg-muted/30 rounded">
        <div class="w-2 h-2 rounded-full bg-green-600"/>
        <span>Strings</span>
      </div>
      <div class="flex items-center gap-2 px-2 py-1 bg-muted/30 rounded">
        <div class="w-2 h-2 rounded-full bg-orange-600"/>
        <span>Dates</span>
      </div>
    </div>
  </div>
</template>
