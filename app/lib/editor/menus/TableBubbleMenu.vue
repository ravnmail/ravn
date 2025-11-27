<script lang="ts" setup>
import type { Editor } from '@tiptap/vue-3'
import { BubbleMenu, isActive } from '@tiptap/vue-3'
import { useLocale } from '@/locales'
import ActionButton from '@/components/ActionButton.vue'
import { sticky } from 'tippy.js'
import type { GetReferenceClientRect } from 'tippy.js'
import HighlightActionButton from '@/extensions/Highlight/components/HighlightActionButton.vue'
import { Separator } from '@/components/ui/separator'

interface Props {
  editor: Editor
}

const props = withDefaults(defineProps<Props>(), {})

const shouldShow = ({ editor }) => {
  return isActive(editor.view.state, 'table')
}
const { t } = useLocale()

function onAddColumnBefore() {
  props.editor.chain().focus().addColumnBefore().run()
}

function onAddColumnAfter() {
  props.editor.chain().focus().addColumnAfter().run()
}

function onDeleteColumn() {
  props.editor.chain().focus().deleteColumn().run()
}

function onAddRowAbove() {
  props.editor.chain().focus().addRowBefore().run()
}

function onAddRowBelow() {
  props.editor.chain().focus().addRowAfter().run()
}

function onDeleteRow() {
  props.editor.chain().focus().deleteRow().run()
}

function onMergeCell() {
  props.editor.chain().focus().mergeCells().run()
}

function onSplitCell() {
  props.editor?.chain().focus().splitCell().run()
}

function onDeleteTable() {
  props.editor.chain().focus().deleteTable().run()
}

function onSetCellBackground(color: string) {
  props.editor.chain().focus().setTableCellBackground(color).run()
}

const getReferenceClientRect: GetReferenceClientRect = () => {
  const {
    view,
    state: {
      selection: { from },
    },
  } = props.editor

  // 获取当前选中的表格节点
  const node = view.domAtPos(from).node as HTMLElement
  if (!node) return new DOMRect(-1000, -1000, 0, 0)
  // 获取表格元素
  const tableWrapper = node?.closest('.tableWrapper')
  if (!tableWrapper) return new DOMRect(-1000, -1000, 0, 0)

  // 获取表格的位置信息
  const rect = tableWrapper.getBoundingClientRect()
  // 返回一个新的 DOMRect，将 bubble menu 定位在表格的上方
  return rect
}
</script>
<template>
  <BubbleMenu
    :editor="editor"
    :should-show="shouldShow"
    :tippy-options="{
      offset: [0, 8],
      popperOptions: {
        modifiers: [{ name: 'flip', enabled: false }],
      },
      maxWidth: 'auto',
      getReferenceClientRect,
      plugins: [sticky],
      sticky: 'popper',
    }"
    :update-delay="0"
    plugin-key="table"
  >
    <div
      class="min-w-32 flex flex-row h-full items-center leading-none gap-0.5 p-2 w-full bg-background rounded-lg shadow-sm border border-border"
    >
      <ActionButton
        :action="onAddColumnBefore"
        :disabled="!editor?.can().addColumnBefore()"
        :tooltip="t('composer.table.menu.insertColumnBefore')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="BetweenHorizonalEnd"
      />
      <ActionButton
        :action="onAddColumnAfter"
        :disabled="!editor?.can().addColumnAfter()"
        :tooltip="t('composer.table.menu.insertColumnAfter')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="BetweenHorizonalStart"
      />
      <ActionButton
        :action="onDeleteColumn"
        :disabled="!editor?.can().deleteColumn()"
        :tooltip="t('composer.table.menu.deleteColumn')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="ColumnDelete"
      />
      <Separator
        class="mx-1 me-2 h-[16px]"
        orientation="vertical"
      />

      <ActionButton
        :action="onAddRowAbove"
        :disabled="!editor?.can().addRowBefore()"
        :tooltip="t('composer.table.menu.insertRowAbove')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="BetweenVerticalEnd"
      />

      <ActionButton
        :action="onAddRowBelow"
        :disabled="!editor?.can().addRowAfter()"
        :tooltip="t('composer.table.menu.insertRowBelow')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="BetweenVerticalStart"
      />
      <ActionButton
        :action="onDeleteRow"
        :disabled="!editor?.can().deleteRow()"
        :tooltip="t('composer.table.menu.deleteRow')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="RowDelete"
      />
      <Separator
        class="mx-1 me-2 h-[16px]"
        orientation="vertical"
      />
      <ActionButton
        :action="onMergeCell"
        :disabled="!editor?.can().mergeCells()"
        :tooltip="t('composer.table.menu.mergeCells')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="TableCellsMerge"
      />
      <ActionButton
        :action="onSplitCell"
        :disabled="!editor?.can().splitCell()"
        :tooltip="t('composer.table.menu.splitCells')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="TableCellsSplit"
      />
      <Separator
        class="mx-1 me-2 h-[16px]"
        orientation="vertical"
      />

      <HighlightActionButton
        :action="onSetCellBackground"
        :editor="editor"
        :tooltip="t('composer.table.menu.setCellsBgColor')"
        :tooltip-options="{
          sideOffset: 15,
        }"
      />
      <ActionButton
        :action="onDeleteTable"
        :disabled="!editor?.can().deleteTable()"
        :tooltip="t('composer.table.menu.deleteTable')"
        :tooltip-options="{
          sideOffset: 15,
        }"
        icon="Trash2"
      />
    </div>
  </BubbleMenu>
</template>
