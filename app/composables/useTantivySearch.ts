import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'
import customParseFormat from 'dayjs/plugin/customParseFormat'

import { invoke } from '@tauri-apps/api/core'
import type { SearchResults } from '~/composables/useSearch'

dayjs.extend(utc)
dayjs.extend(customParseFormat)

export interface SearchField {
  type: 'text' | 'boolean' | 'datetime'
  label: string
  description?: string
}

export interface SearchFields {
  [key: string]: SearchField
}

export interface QueryToken {
  type: 'field' | 'operator' | 'keyword' | 'string' | 'date_range' | 'group'
  value: string
  fieldName?: string
  fieldType?: SearchField['type']
}

export interface SearchOptions {
  query: string
  accountId?: string
  folderId?: string
  unreadOnly?: boolean
  flaggedOnly?: boolean
  limit?: number
  offset?: number
}

interface SearchResponse {
  results: any[]
  total: number
}

export const useTantivySearch = (fields: SearchFields) => {
  const query = ref<string>('')
  const loading = ref<boolean>(false)
  const error = ref<string | null>(null)
  const results = ref<any[]>([])
  const totalResults = ref<number>(0)

  const parseQuery = (q: string): QueryToken[] => {
    if (!q.trim()) return []

    const tokens: QueryToken[] = []
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
        tokens.push({
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
        tokens.push({
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
        const fieldName = fieldMatch[1].toLowerCase()
        const fieldDef = fields[fieldName]

        tokens.push({
          type: 'field',
          value: matched,
          fieldName,
          fieldType: fieldDef?.type,
        })
        i += matched.length
        continue
      }

      const opPattern = /^(AND|OR|NOT)(?=\s|$)/i
      const opMatch = q.slice(i).match(opPattern)
      if (opMatch) {
        tokens.push({
          type: 'operator',
          value: opMatch[1].toUpperCase(),
        })
        i += opMatch[1].length
        continue
      }

      if (ch === '(' || ch === ')') {
        tokens.push({
          type: 'group',
          value: ch,
        })
        i++
        continue
      }

      const wordPattern = /^[^\s():\[\]"]+/
      const wordMatch = q.slice(i).match(wordPattern)
      if (wordMatch) {
        tokens.push({
          type: 'keyword',
          value: wordMatch[0],
        })
        i += wordMatch[0].length
        continue
      }

      i++
    }

    return tokens
  }

  const validateQuery = (q: string): { valid: boolean; error?: string } => {
    const MAX_LENGTH = 2000
    const MAX_OR_CLAUSES = 50
    const MAX_WILDCARDS = 5

    if (q.length > MAX_LENGTH) {
      return { valid: false, error: `Query exceeds max length (${MAX_LENGTH} chars)` }
    }

    const orCount = (q.match(/\bOR\b/gi) || []).length
    if (orCount > MAX_OR_CLAUSES) {
      return { valid: false, error: `Too many OR clauses (max ${MAX_OR_CLAUSES})` }
    }

    const wildcardCount = (q.match(/\*/g) || []).length
    if (wildcardCount > MAX_WILDCARDS) {
      return { valid: false, error: `Too many wildcards (max ${MAX_WILDCARDS})` }
    }

    let parenCount = 0
    for (const char of q) {
      if (char === '(') parenCount++
      if (char === ')') parenCount--
      if (parenCount < 0) {
        return { valid: false, error: 'Unbalanced parentheses' }
      }
    }
    if (parenCount !== 0) {
      return { valid: false, error: 'Unbalanced parentheses' }
    }

    let quoteCount = 0
    for (let i = 0; i < q.length; i++) {
      if (q[i] === '"' && (i === 0 || q[i - 1] !== '\\')) {
        quoteCount++
      }
    }
    if (quoteCount % 2 !== 0) {
      return { valid: false, error: 'Unbalanced quotes' }
    }

    return { valid: true }
  }

  const buildBooleanValue = (value: boolean): string => {
    return value ? 'true' : 'false'
  }

  const parseNaturalDate = (input: string): string | null => {
    const now = dayjs()
    const lower = input.toLowerCase().trim()

    const patterns: Record<string, () => string> = {
      'today': () => now.format('YYYY-MM-DD'),
      'yesterday': () => now.subtract(1, 'day').format('YYYY-MM-DD'),
      'tomorrow': () => now.add(1, 'day').format('YYYY-MM-DD'),
      'last week': () => now.subtract(1, 'week').format('YYYY-MM-DD'),
      'this week': () => now.startOf('week').format('YYYY-MM-DD'),
      'last month': () => now.subtract(1, 'month').format('YYYY-MM-DD'),
      'this month': () => now.startOf('month').format('YYYY-MM-DD'),
      'last year': () => now.subtract(1, 'year').format('YYYY-MM-DD'),
      'this year': () => now.startOf('year').format('YYYY-MM-DD'),
    }

    if (patterns[lower]) {
      return patterns[lower]()
    }

    const formats = [
      'YYYY-MM-DD',
      'DD.MM.YYYY',
      'DD/MM/YYYY',
      'MM/DD/YYYY',
      'YYYY/MM/DD',
    ]

    for (const format of formats) {
      const parsed = dayjs(input, format, true)
      if (parsed.isValid()) {
        return parsed.format('YYYY-MM-DD')
      }
    }

    return null
  }

  const buildDateRange = (startStr: string, endStr: string): string | null => {
    const start = parseNaturalDate(startStr)
    const end = parseNaturalDate(endStr)

    if (!start || !end) return null

    const startISO = dayjs(start).utc().format('YYYY-MM-DDTHH:mm:ss[Z]')
    const endISO = dayjs(end).utc().format('YYYY-MM-DDTHH:mm:ss[Z]')

    return `[${startISO} TO ${endISO}]`
  }

  const buildTextValue = (value: string): string => {
    if (value.includes(' ')) {
      return `"${value}"`
    }
    return value
  }

  const buildFieldToken = (fieldName: string, value: string): string | null => {
    const field = fields[fieldName.toLowerCase()]
    if (!field) return null

    switch (field.type) {
      case 'boolean':
        return `${fieldName}:${buildBooleanValue(value === 'true')}`
      case 'datetime':
        if (value.startsWith('[')) {
          return `${fieldName}:${value}`
        }
        return null
      case 'text':
      default:
        return `${fieldName}:${buildTextValue(value)}`
    }
  }

  const buildQuery = (tokens: Array<{ type: string; fieldName?: string; value: string }>): string => {
    return tokens.map(token => token.value).join(' ')
  }

  const highlightSyntax = (queryString: string): Array<{ text: string; type: string }> => {
    const tokens = parseQuery(queryString)
    return tokens.map(token => ({
      text: token.value,
      type: token.type,
    }))
  }

  const search = async (options: SearchOptions): Promise<void> => {
    const validation = validateQuery(options.query)
    if (!validation.valid) {
      error.value = validation.error || 'Invalid query'
      return
    }

    loading.value = true
    error.value = null

    try {
      const response = await invoke<SearchResults>('search_emails', {
        query: options.query,
        accountId: options.accountId,
        folderId: options.folderId,
        limit: options.limit ?? 100,
        offset: options.offset ?? 0,
      })

      results.value = response.emails || []
      totalResults.value = response.total || 0
      query.value = options.query
    } catch (err: any) {
      error.value = err.message || 'Search failed'
      results.value = []
      totalResults.value = 0
    } finally {
      loading.value = false
    }
  }

  const getExamples = (): Array<{ query: string; description: string }> => {
    return [
      { query: 'from:john@example.com', description: 'Search by sender' },
      { query: 'subject:"urgent"', description: 'Search by subject' },
      { query: 'is_read:true', description: 'Read emails only' },
      { query: 'from:john AND subject:meeting', description: 'Complex query with AND' },
      { query: '[2023-01-01 TO 2023-12-31]', description: 'Date range search' },
    ]
  }

  const getFieldSuggestions = (usedFields: string[] = []): string[] => {
    return Object.keys(fields).filter(name => !usedFields.includes(name))
  }

  const getUsedFields = (tokens: QueryToken[]): string[] => {
    return tokens
      .filter(t => t.type === 'field' && t.fieldName)
      .map(t => t.fieldName!)
      .filter((v, i, a) => a.indexOf(v) === i)
  }

  const hasError = computed(() => !!error.value)
  const isLoading = computed(() => loading.value)
  const isEmpty = computed(() => results.value.length === 0)
  const resultsCount = computed(() => results.value.length)

  return {
    query,
    loading,
    error,
    results,
    totalResults,

    hasError,
    isLoading,
    isEmpty,
    resultsCount,

    parseQuery,
    validateQuery,
    parseNaturalDate,
    buildBooleanValue,
    buildDateRange,
    buildTextValue,
    buildFieldToken,
    search,
    getExamples,
    getFieldSuggestions,
    getUsedFields,
  }
}
