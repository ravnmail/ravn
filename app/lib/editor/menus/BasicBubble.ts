import { deleteSelection } from '@tiptap/pm/commands'
import type { Editor } from '@tiptap/vue-3'
import ActionButton from '@/components/ActionButton.vue'

import type { ButtonViewParams, ButtonViewReturn, ExtensionNameKeys } from '@/types/composer'

type BubbleImageOrVideoSizeType = 'size-small' | 'size-medium' | 'size-large'
type BubbleImageType = `video-${BubbleImageOrVideoSizeType}` | 'image' | 'image-aspect-ratio' | 'remove'
type BubbleVideoType = 'video' | 'remove'
type BubbleAllType = BubbleImageType | BubbleVideoType | ExtensionNameKeys | 'divider' | (string & {})

export type NodeTypeKey = 'image' | 'text' | 'video'
export type BubbleTypeMenu = Partial<Record<NodeTypeKey, BubbleMenuItem[]>>
export type NodeTypeMenu = Partial<Record<NodeTypeKey, BubbleAllType[]>>

export interface BubbleMenuItem extends ButtonViewReturn {
  type: BubbleAllType
}

interface BubbleView<T = any> {
  /**
   * Generates a bubble menu based on the provided options.
   * @param {ButtonViewParams<T>} options - The options for generating the bubble menu.
   * @returns {BubbleTypeMenu} The generated bubble menu.
   */
  (options: ButtonViewParams<T>): BubbleTypeMenu
}

/**
 * Represents the options for configuring bubbles.
 * @interface BubbleOptions
 * @template T
 */
export interface BubbleOptions<T> {
  list: NodeTypeMenu
  defaultBubbleList: typeof defaultBubbleList
  button: BubbleView<T>
}

export const defaultBubbleList = (editor: Editor): BubbleMenuItem[] => [
  {
    type: 'remove',
    component: ActionButton,
    componentProps: {
      tooltip: 'composer.remove',
      icon: 'trash-2',
      action: () => {
        const { state, dispatch } = editor.view
        deleteSelection(state, dispatch)
      },
    },
  },
]

/**
 * bubble menu
 * @template T
 * @param {NodeTypeMenu} list
 * @param {BubbleMenuItem[]} defaultList
 * @param {ButtonViewParams<T>} { editor, extension, t }
 * @return {*}  {BubbleTypeMenu}
 */
export const generateBubbleTypeMenu = <T = any>(
  list: NodeTypeMenu,
  defaultList: BubbleMenuItem[],
  { editor, extension, t }: ButtonViewParams<T>
): BubbleTypeMenu => {
  const { extensions = [] } = editor.extensionManager
  const items: BubbleTypeMenu = {}

  for (const node of Object.keys(list)) {
    const nodeType = list[node as NodeTypeKey]
    if (!nodeType) continue

    const _items: BubbleMenuItem[] = []

    for (const ext of nodeType) {
      if (ext === 'divider') {
        const lastItem = _items[_items.length - 1]
        if (lastItem?.type === 'divider') continue

        _items.push({
          type: 'divider',
          component: undefined,
          componentProps: {},
        })
        continue
      }

      const find = defaultList.find(k => k.type === ext)
      if (find) {
        _items.push({
          ...find,
          componentProps: {
            ...find.componentProps,
            tooltip: find.componentProps.tooltip ? t(find.componentProps.tooltip) : undefined,
          },
          componentSlots: find.componentSlots,
        })
        continue
      }

      const findExt = extensions.find(k => k.name === ext)
      if (findExt) {
        const { button } = findExt.options as any
        const _button: ButtonViewReturn = button({
          editor,
          extension: findExt,
          t,
        })

        _items.push({
          type: ext,
          component: _button.component,
          componentProps: _button.componentProps,
          componentSlots: _button.componentSlots,
        })
        continue
      }
    }

    const lastItem = _items[_items.length - 1]
    const firstItem = _items[0]

    if (lastItem?.type === 'divider') _items.pop()
    if (firstItem?.type === 'divider') _items.shift()

    items[node as NodeTypeKey] = _items
  }

  return items
}
