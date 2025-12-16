<script lang="ts" setup>
import { computed, reactive } from 'vue'
import { FormField } from '~/components/ui/form'
import { Input } from '~/components/ui/input'
import { Button } from '~/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '~/components/ui/table'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '~/components/ui/select'
import { Textarea } from '~/components/ui/textarea'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import { Switch } from '~/components/ui/switch'

type InputType =
  'text'
  | 'number'
  | 'email'
  | 'password'
  | 'select'
  | 'checkbox'
  | 'textarea'
  | 'date'
  | 'time'
  | 'datetime-local'
  | 'url'
  | 'tel'

interface SelectOption {
  value: string | number
  label: string
  disabled?: boolean
}

interface ColumnDefinition {
  key: string
  label: string | CleanTranslation
  type: InputType
  placeholder?: string | CleanTranslation
  required?: boolean
  editable?: boolean
  creatable?: boolean
  readonly?: boolean
  width?: string
  maxLength?: number
  min?: number
  max?: number
  step?: number
  rows?: number
  options?: SelectOption[]
  validate?: (value: unknown) => boolean | string
  transform?: (value: unknown) => unknown
  defaultValue?: unknown
  description?: string | CleanTranslation
}

interface Props {
  modelValue: Record<string, unknown>[]
  name: string
  label: string | CleanTranslation
  description?: string | CleanTranslation
  columns: ColumnDefinition[]
  disabled?: boolean
  required?: boolean
  emptyMessage?: string | CleanTranslation
  addButtonText?: string | CleanTranslation
  showAddButton?: boolean
  showRemoveButton?: boolean
  sortable?: boolean
  maxItems?: number
  minItems?: number
  showEmptyPlaceholder?: boolean
  validateRow?: (row: Record<string, unknown>) => boolean | string
}

const props = withDefaults(defineProps<Props>(), {
  emptyMessage: 'No items added yet',
  addButtonText: 'Add Item',
  description: undefined,
  showAddButton: true,
  showRemoveButton: true,
  showEmptyPlaceholder: false,
  sortable: false,
  maxItems: undefined,
  minItems: 0,
  validateRow: undefined,
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: Record<string, unknown>[]): void
  (e: 'add', item: Record<string, unknown>): void
  (e: 'remove' | 'update', index: number, item: Record<string, unknown>): void
}>()

const newItem = reactive<Record<string, unknown>>({})

const initializeNewItem = (): void => {
  props.columns.forEach(column => {
    if (column.creatable !== false) {
      newItem[column.key] = column.defaultValue ?? getDefaultValueForType(column.type)
    }
  })
}

const getDefaultValueForType = (type: InputType): string | boolean | number => {
  switch (type) {
    case 'number':
      return 0
    case 'checkbox':
      return false
    default:
      return ''
  }
}

initializeNewItem()

const items = computed({
  get: (): Record<string, unknown>[] => props.modelValue || [],
  set: (value: Record<string, unknown>[]) => emit('update:modelValue', value)
})

const creatableColumns = computed((): ColumnDefinition[] =>
  props.columns.filter(col => col.creatable !== false)
)

const validateField = (column: ColumnDefinition, value: unknown): boolean => {
  if (column.required && (value === '' || value === null || value === undefined)) {
    return false
  }

  if (column.validate) {
    const result = column.validate(value)
    return result === true
  }

  return true
}

const validateNewItem = (): boolean => {
  for (const column of creatableColumns.value) {
    if (!validateField(column, newItem[column.key])) {
      return false
    }
  }

  if (props.validateRow) {
    const result = props.validateRow(newItem)
    return result === true
  }

  return true
}

const canAddItem = computed((): boolean => {
  if (props.disabled) return false
  if (props.maxItems && items.value.length >= props.maxItems) return false
  return validateNewItem()
})

const addItem = (): void => {
  if (!canAddItem.value) return

  const itemToAdd = { ...newItem }

  props.columns.forEach(column => {
    if (column.transform && itemToAdd[column.key] !== undefined) {
      itemToAdd[column.key] = column.transform(itemToAdd[column.key])
    }
  })

  items.value = [...items.value, itemToAdd]
  emit('add', itemToAdd)

  initializeNewItem()
}

const removeItem = (index: number): void => {
  if (props.disabled) return
  if (props.minItems && items.value.length <= props.minItems) return

  const itemToRemove = items.value[index]
  items.value = items.value.filter((_, i) => i !== index)
  emit('remove', index, itemToRemove)
}

const updateItem = (index: number, key: string, value: unknown): void => {
  if (props.disabled) return

  const column = props.columns.find(col => col.key === key)
  if (!column || column.readonly) return

  const updatedItems = [...items.value]

  const finalValue = column.transform ? column.transform(value) : value

  updatedItems[index] = { ...updatedItems[index], [key]: finalValue }
  items.value = updatedItems
  emit('update', index, updatedItems[index])
}

const handleKeyPress = (event: KeyboardEvent): void => {
  if (event.key === 'Enter') {
    event.preventDefault()
    addItem()
  }
}

</script>

<template>
  <FormField
    :description="description"
    :label="label"
    :name="name"
    :required="required"
  >
    <div class="space-y-4">
      <Table class="bg-elevated rounded-md overflow-hidden">
        <TableHeader>
          <TableRow>
            <TableHead
              v-if="sortable"
              class="w-[40px]"
            />

            <TableHead
              v-for="column in columns"
              :key="column.key"
              :style="column.width ? { width: column.width } : undefined"
            >
              {{ String(column.label) }}
              <span
                v-if="column.required"
                class="text-destructive ml-1"
              >*</span>
            </TableHead>

            <TableHead
              v-if="showRemoveButton"
              class="w-[50px]"
            />
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="(item, index) in items"
            :key="`item-${index}`"
          >
            <TableCell v-if="sortable">
              <Button
                :disabled="disabled"
                class="cursor-grab"
                size="icon"
                type="button"
                variant="ghost"
              >
                <Icon name="lucide:grip-vertical"/>
              </Button>
            </TableCell>

            <TableCell
              v-for="column in columns"
              :key="column.key"
            >
              <Input
                v-if="column.type === 'text' || column.type === 'email' || column.type === 'password' || column.type === 'url' || column.type === 'tel'"
                :disabled="disabled || column.readonly || (column.editable === false)"
                :maxlength="column.maxLength"
                :model-value="String(item[column.key] || '')"
                :placeholder="String(column.placeholder || '')"
                :type="column.type"
                @update:model-value="updateItem(index, column.key, $event)"
              />

              <Input
                v-else-if="column.type === 'number'"
                :disabled="disabled || column.readonly || (column.editable === false)"
                :max="column.max"
                :min="column.min"
                :model-value="String(item[column.key] || '')"
                :placeholder="String(column.placeholder || '')"
                :step="column.step"
                type="number"
                @update:model-value="updateItem(index, column.key, Number($event))"
              />

              <Input
                v-else-if="['date', 'time', 'datetime-local'].includes(column.type)"
                :disabled="disabled || column.readonly || (column.editable === false)"
                :model-value="String(item[column.key] || '')"
                :type="column.type"
                @update:model-value="updateItem(index, column.key, $event)"
              />

              <Textarea
                v-else-if="column.type === 'textarea'"
                :disabled="disabled || column.readonly || (column.editable === false)"
                :maxlength="column.maxLength"
                :model-value="String(item[column.key] || '')"
                :placeholder="String(column.placeholder || '')"
                :rows="column.rows || 3"
                @update:model-value="updateItem(index, column.key, $event)"
              />

              <Select
                v-else-if="column.type === 'select'"
                :disabled="disabled || column.readonly || (column.editable === false)"
                :model-value="String(item[column.key] || '')"
                @update:model-value="updateItem(index, column.key, $event)"
              >
                <SelectTrigger>
                  <SelectValue :placeholder="String(column.placeholder || '')"/>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="option in column.options"
                    :key="option.value"
                    :disabled="option.disabled"
                    :value="String(option.value)"
                  >
                    {{ option.label }}
                  </SelectItem>
                </SelectContent>
              </Select>

              <Switch
                v-else-if="column.type === 'checkbox'"
                :checked="Boolean(item[column.key])"
                :disabled="disabled || column.readonly || (column.editable === false)"
                @update:checked="updateItem(index, column.key, $event)"
              />

              <span
                v-else-if="column.readonly || column.editable === false"
                class="text-sm text-muted-foreground"
              >
                {{ item[column.key] }}
              </span>
            </TableCell>

            <TableCell
              v-if="showRemoveButton"
              class="flex"
            >
              <Button
                :disabled="disabled || (minItems && items.length <= minItems)"
                class="ml-auto"
                size="icon"
                type="button"
                variant="ghost"
                @click="removeItem(index)"
              >
                <Icon name="lucide:trash-2"/>
                <span class="sr-only">{{ $t('actions.remove') }}</span>
              </Button>
            </TableCell>
          </TableRow>

          <TableRow v-if="showEmptyPlaceholder && items.length === 0">
            <TableCell
              :colspan="columns.length + (sortable ? 1 : 0) + (showRemoveButton ? 1 : 0)"
              class="text-center py-6 text-muted-foreground"
            >
              {{ String(emptyMessage) }}
            </TableCell>
          </TableRow>

          <TableRow v-if="showAddButton && (!maxItems || items.length < maxItems)">
            <TableCell v-if="sortable"/>

            <TableCell
              v-for="column in columns"
              :key="`new-${column.key}`"
            >
              <template v-if="column.creatable !== false">
                <Input
                  v-if="column.type === 'text' || column.type === 'email' || column.type === 'password' || column.type === 'url' || column.type === 'tel'"
                  v-model="newItem[column.key]"
                  :disabled="disabled"
                  :maxlength="column.maxLength"
                  :placeholder="String(column.placeholder || '')"
                  :type="column.type"
                  @keydown="handleKeyPress"
                />

                <Input
                  v-else-if="column.type === 'number'"
                  v-model="newItem[column.key]"
                  :disabled="disabled"
                  :max="column.max"
                  :min="column.min"
                  :placeholder="String(column.placeholder || '')"
                  :step="column.step"
                  type="number"
                  @keydown="handleKeyPress"
                />

                <Input
                  v-else-if="['date', 'time', 'datetime-local'].includes(column.type)"
                  v-model="newItem[column.key]"
                  :disabled="disabled"
                  :type="column.type"
                  @keydown="handleKeyPress"
                />

                <Textarea
                  v-else-if="column.type === 'textarea'"
                  v-model="newItem[column.key]"
                  :disabled="disabled"
                  :maxlength="column.maxLength"
                  :placeholder="String(column.placeholder || '')"
                  :rows="column.rows || 3"
                />

                <Select
                  v-else-if="column.type === 'select'"
                  v-model="newItem[column.key]"
                  :disabled="disabled"
                >
                  <SelectTrigger>
                    <SelectValue :placeholder="String(column.placeholder || '')"/>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem
                      v-for="option in column.options"
                      :key="option.value"
                      :disabled="option.disabled"
                      :value="String(option.value)"
                    >
                      {{ option.label }}
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Switch
                  v-else-if="column.type === 'checkbox'"
                  v-model="newItem[column.key]"
                  :disabled="disabled"
                />
              </template>
            </TableCell>

            <TableCell v-if="showRemoveButton">
              <Button
                :disabled="!canAddItem"
                size="sm"
                type="button"
                @click="addItem"
              >
                <Icon name="lucide:plus"/>
                <span>{{ String(addButtonText) }}</span>
              </Button>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

      <div
        v-if="minItems && items.length < minItems"
        class="text-sm text-destructive"
      >
        {{ $t('validation.minItems', { min: minItems, current: items.length }) }}
      </div>

      <div
        v-if="maxItems && items.length >= maxItems"
        class="text-sm text-muted-foreground"
      >
        {{ $t('validation.maxItems', { max: maxItems }) }}
      </div>
    </div>
  </FormField>
</template>