<script lang="ts" setup>
import { computed } from 'vue'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider, SimpleTooltip } from '@/components/ui/tooltip'
import type { Editor } from '@tiptap/vue-3'
import ActionDropdownButtonSplit from '@/components/ActionDropdownButtonSplit.vue'
import { MenuItem } from '@/components/ui/menu'
import type { ButtonViewReturnComponentProps } from '@/types/composer'

interface OrderedListOption {
  label: string
  value:
    | 'decimal'
    | 'decimal-leading-zero'
    | 'lower-roman'
    | 'upper-roman'
    | 'lower-latin'
    | 'upper-latin'
    | 'trad-chinese-informal'
    | 'simp-chinese-formal'
}

interface Props {
  editor: Editor
  disabled?: boolean
  tooltip?: string
  shortcutKeys?: string[]
  isActive?: ButtonViewReturnComponentProps['isActive']
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  tooltip: '',
  shortcutKeys: undefined,
  isActive: undefined,
})

const { t } = useI18n()

const OrderedListOptions: OrderedListOption[] = [
  { label: 'editor.orderedlist.decimal.tooltip', value: 'decimal' },
  {
    label: 'editor.orderedlist.decimalLeadingZero.tooltip',
    value: 'decimal-leading-zero',
  },
  { label: 'editor.orderedlist.lowerRoman.tooltip', value: 'lower-roman' },
  { label: 'editor.orderedlist.upperRoman.tooltip', value: 'upper-roman' },
  { label: 'editor.orderedlist.lowerLatin.tooltip', value: 'lower-latin' },
  { label: 'editor.orderedlist.upperLatin.tooltip', value: 'upper-latin' },
  {
    label: 'editor.orderedlist.tradChineseInformal.tooltip',
    value: 'trad-chinese-informal',
  },
  {
    label: 'editor.orderedlist.simpChineseFormal.tooltip',
    value: 'simp-chinese-formal',
  },
]

const active = computed(() => {
  if (props.editor?.isActive('orderedList')) {
    return props.editor.getAttributes('orderedList').listType
  }
  return 'decimal' // 默认值
})

function toggleOrderedList(item: OrderedListOption) {
  if (props.editor.isActive('orderedList')) {
    if (props.editor.getAttributes('orderedList').listType === item.value) {
      props.editor.chain()?.toggleOrderedList().run()
    } else {
      props.editor.chain()?.updateAttributes('orderedList', { listType: item.value }).run()
    }
  } else {
    props.editor.chain().focus().toggleOrderedList().updateAttributes('orderedList', { listType: item.value }).run()
  }
}
</script>

<template>
  <ActionDropdownButtonSplit
    :action="toggleOrderedList"
    :disabled="disabled"
    :is-active="isActive"
    :shortcut-keys="shortcutKeys"
    :tooltip="tooltip"
    class="min-w-4 w-full grid grid-cols-3 gap-1"
  >
    <SimpleTooltip
      v-for="item in OrderedListOptions"
      :key="item.value"
      :delay-duration="0"
      :tooltip="t(item.label)"
      side="bottom"
    >
      <MenuItem
        class="p-0!"
        @click="toggleOrderedList(item)"
      >
        <div
          :class="[active === item.value ? 'bg-selection border ' : '']"
          class="h-10 flex flex-col w-12 border-input rounded-sm border overflow-clip"
        >
          <ol
            :style="{ listStyleType: item.value, lineHeight: 1 }"
            class="text-muted pl-3 flex-1 list-inside"
          >
            <li
              v-for="i in 3"
              :key="i"
            />
          </ol>
        </div>
      </MenuItem>
    </SimpleTooltip>
  </ActionDropdownButtonSplit>
</template>
