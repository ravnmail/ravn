<script lang="ts" setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { DragHandlePlugin } from '@/lib/editor/DragHandle'
import { Button } from '@/components/ui/button'
import type { Node } from '@tiptap/pm/model'
import type { Editor } from '@tiptap/vue-3'
import type { NodeSelection, Plugin, PluginKey } from '@tiptap/pm/state'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { SimpleTooltip } from '@/components/ui/tooltip'
import { IndentProps, setNodeIndentMarkup } from '@/lib/utils/indent'
import { getShortcutKeys } from '@/lib/utils/platform'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'

type PluginRefType = Plugin<{
  locked: boolean
}>

interface Props {
  className?: string
  editor: Editor
  disabled?: boolean
  pluginKey?: PluginKey | string
}

const props = withDefaults(defineProps<Props>(), {
  className: 'drag-handle',
  disabled: false,
  pluginKey: 'ContentItemMenu',
})
const { t } = useI18n()

const dragElement = ref(null)
const pluginRef = ref<PluginRefType | null>(null)
const currentNode = ref<Node | null>(null)
const currentNodePos = ref<number>(-1)
const menuOpen = ref(false)

onMounted(() => {
  if (dragElement.value && !props.editor.isDestroyed) {
    pluginRef.value = DragHandlePlugin({
      editor: props.editor,
      element: dragElement.value,
      pluginKey: props.pluginKey,
      tippyOptions: {
        offset: [2, 4],
        zIndex: 50,
        appendTo: () => document.body,
        moveTransition: 'transform 0.15s ease-out',
      },
      onNodeChange: handleNodeChange,
    })

    props.editor.registerPlugin(pluginRef.value)
  }
})

onUnmounted(() => {
  if (pluginRef.value) {
    props.editor.unregisterPlugin(props.pluginKey)
  }
})

function resetTextFormatting() {
  const chain = props.editor.chain()
  chain.setNodeSelection(currentNodePos.value).unsetAllMarks()
  if (currentNode.value?.type.name !== 'paragraph') {
    chain.setParagraph()
  }
  chain.run()
}

function copyNodeToClipboard() {
  props.editor.commands.setNodeSelection(currentNodePos.value)
  document.execCommand('copy')
}

function duplicateNode() {
  props.editor.commands.setNodeSelection(currentNodePos.value)
  const { $anchor } = props.editor.state.selection
  const selectedNode = $anchor.node(1) || (props.editor.state.selection as NodeSelection).node
  props.editor
    .chain()
    .setMeta('hideDragHandle', true)
    .insertContentAt(currentNodePos.value + (currentNode.value?.nodeSize || 0), selectedNode.toJSON())
    .focus()
    .run()
}

function increaseIndent() {
  const indentTr = setNodeIndentMarkup(props.editor.state.tr, currentNodePos.value, 1)
  indentTr.setMeta('hideDragHandle', true)
  props.editor.view.dispatch && props.editor.view.dispatch(indentTr)
}

function decreaseIndent() {
  const tr = setNodeIndentMarkup(props.editor.state.tr, currentNodePos.value, -1)
  props.editor.view.dispatch && props.editor.view.dispatch(tr)
}

function deleteNode() {
  props.editor.chain().setMeta('hideDragHandle', true).setNodeSelection(currentNodePos.value).deleteSelection().run()
}

function handleNodeChange(e) {
  if (e.node) {
    currentNode.value = e.node
  }
  currentNodePos.value = e.pos
}

function handleAdd(event: MouseEvent) {
  if (!props.editor.isEditable) {
    return
  }
  if (currentNodePos.value !== -1) {
    const currentNodeSize = currentNode.value?.nodeSize || 0
    const insertPos = currentNodePos.value + currentNodeSize

    const currentNodeIsEmptyParagraph =
      currentNode.value?.type.name === 'paragraph' && currentNode.value?.content?.size === 0
    const focusPos = currentNodeIsEmptyParagraph ? currentNodePos.value + 2 : insertPos + 2
    props.editor
      .chain()
      .command(({ dispatch, tr, state }) => {
        if (dispatch) {
          if (currentNodeIsEmptyParagraph) {
            tr.insertText('/', currentNodePos.value, currentNodePos.value + 1)
          } else {
            tr.insert(insertPos, state.schema.nodes.paragraph.create(null, [state.schema.text('/')]))
          }

          return dispatch(tr)
        }

        return true
      })
      .focus(focusPos)
      .run()
  }
}

watch(
  () => menuOpen.value,
  val => {
    if (val) {
      props.editor.commands.setHighlightParagraph(currentNodePos.value)
      props.editor.commands.setMeta('lockDragHandle', true)
    } else {
      props.editor.commands.clearHighlightParagraph()
      props.editor.commands.setMeta('lockDragHandle', false)
    }
  }
)

watch(
  () => props.editor.isDestroyed,
  isDestroyed => {
    if (isDestroyed && pluginRef.value) {
      props.editor.unregisterPlugin(props.pluginKey)
      pluginRef.value = null
    }
  }
)
</script>
<template>
  <div
    v-show="!disabled"
    ref="dragElement"
    :class="className"
  >
    <div class="flex items-start">
      <SimpleTooltip
        :tooltip-markdown="t('composer.draghandle.add')"
        side="bottom"
      >
        <Button
          :disabled="disabled"
          class="size-5 rounded-sm"
          size="icon"
          variant="ghost"
          @click="handleAdd"
        >
          <Icon
            class="text-muted"
            name="lucide:plus"
          />
        </Button>
      </SimpleTooltip>
      <DropdownMenu v-model:open="menuOpen">
        <DropdownMenuTrigger
          :disable="disabled"
          as-child
        >
          <SimpleTooltip
            :tooltip-markdown="t('composer.draghandle.drag')"
            side="bottom"
          >
            <Button
              class="size-5 cursor-grab rounded-sm"
              size="icon"
              variant="ghost"
            >
              <Icon
                class="text-muted"
                name="lucide:grip-vertical"
              />
            </Button>
          </SimpleTooltip>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="start"
          class="w-48"
          side="bottom"
        >
          <DropdownMenuItemRich
            :label="t('composer.remove')"
            icon="lucide:trash-2"
            @click="deleteNode"
          />
          <DropdownMenuItemRich
            :label="t('composer.clear.tooltip')"
            icon="lucide:paint-roller"
            @click="resetTextFormatting"
          />
          <DropdownMenuItemRich
            :label="t('composer.copyToClipboard')"
            icon="lucide:clipboard"
            @click="copyNodeToClipboard"
          />
          <DropdownMenuItemRich
            :label="t('composer.copy')"
            class="flex gap-3"
            icon="lucide:copy"
            @click="duplicateNode"
          />
          <DropdownMenuSeparator/>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger>
              <Icon name="lucide:indent-increase"/>
              <span>{{ t('composer.indent.tooltip') }}</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuPortal>
              <DropdownMenuSubContent>
                <DropdownMenuItemRich
                  :disabled="currentNode?.attrs?.indent >= IndentProps.max"
                  :label="t('composer.indent.tooltip')"
                  :shortcut="getShortcutKeys(['Tab'])"
                  icon="lucide:indent-increase"
                  @click="increaseIndent"
                />

                <DropdownMenuItemRich
                  :disabled="currentNode?.attrs?.indent <= IndentProps.min"
                  :label="t('composer.outdent.tooltip')"
                  :shortcut="getShortcutKeys(['Shift, Tab'])"
                  icon="lucide:indent-decrease"
                  @click="decreaseIndent"

                />
              </DropdownMenuSubContent>
            </DropdownMenuPortal>
          </DropdownMenuSub>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  </div>
</template>
