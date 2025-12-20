<script lang="ts" setup>

import EmailLabel from '~/components/ui/EmailLabel.vue'
import {
  ListboxRoot,
  ListboxFilter,
  PopoverAnchor,
  useFilter,
  ListboxContent,
  ListboxItemIndicator
} from 'reka-ui'
import { ListboxItem } from '~/components/ui/listbox'
import { Popover, PopoverContent, PopoverTrigger } from '~/components/ui/popover'
import { Button } from '~/components/ui/button'
import { TagsInput, TagsInputInput, TagsInputItem, TagsInputItemDelete } from '~/components/ui/tags-input'

const { t } = useI18n()
const { labels } = useLabels()

const props = withDefaults(defineProps<{
  modelValue?: string[] | undefined
}>(), {
  modelValue: () => [],
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string[]): void
}>()

const selectedLabels = computed(() => {
  return props.modelValue.map(id => labels.value?.find(l => l.id === id))
})

const searchTerm = ref('')
const open = ref(false)
const { contains } = useFilter({ sensitivity: 'base' })

const filteredLabels = computed(() => {
  if (!searchTerm.value) {
    return labels.value || []
  }

  return labels.value?.filter(label => contains(label.name, searchTerm.value)) || []
})

watch(searchTerm, (f) => {
  if (f) {
    open.value = true
  }
})
</script>

<template>
  <Popover v-model:open="open">
    <ListboxRoot
      :model-value="modelValue"
      highlight-on-hover
      multiple
      @update:model-value="emit('update:modelValue', $event)"
    >
      <label class="text-sm font-medium">{{ t('components.viewWizard.customize.includeLabels') }}</label>
      <PopoverAnchor>
        <TagsInput
          :model-value="modelValue"
          class="pr-0"
          @update:model-value="emit('update:modelValue', $event)"
        >
          <TagsInputItem
            v-for="label in selectedLabels"
            :key="label.id"
            :value="label.id"
          >
            <EmailLabel
              v-bind="label"
            >
              <TagsInputItemDelete class="mr-0"/>
            </EmailLabel>
          </TagsInputItem>
          <ListboxFilter
            v-model="searchTerm"
            as-child
          >
            <TagsInputInput
              :placeholder="t('components.viewWizard.customize.filterLabels')"
              @keydown.enter.prevent
              @keydown.down="open = true"
            />
          </ListboxFilter>
          <PopoverTrigger as-child>
            <Button
              variant="ghost"
            >
              <Icon name="lucide:chevron-down"/>
            </Button>
          </PopoverTrigger>
        </TagsInput>
      </PopoverAnchor>
      <PopoverContent
        @open-auto-focus.prevent
      >
        <ListboxContent
          class="max-h-75 scroll-py-1 overflow-x-hidden overflow-y-auto empty:after:content-['No_options'] empty:p-1 empty:after:block"
          tabindex="0"
        >
          <ListboxItem
            v-for="label in filteredLabels"
            :key="label.id"
            :value="label.id"
            @select="searchTerm = ''"
          >
            <EmailLabel
              class="font-medium"
              v-bind="label"
            />
          </ListboxItem>
        </ListboxContent>
      </PopoverContent>
    </ListboxRoot>
  </popover>
</template>