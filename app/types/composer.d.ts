import type { CleanTranslation, Params  } from 'nuxt-i18n-micro-types'
import type { RouteLocationNormalizedLoaded } from '#vue-router'

export interface ButtonViewReturnComponentProps {
  action?: (value?: unknown) => unknown
  isActive?: () => boolean
  icon?: string
  customClass?: string
  disabled?: boolean
  shortcutKeys?: string[]
  tooltip?: CleanTranslation | string
  [x: string]: unknown
}

export type ExtensionNameKeys =
  | 'bold'
  | 'italic'
  | 'underline'
  | 'strike'
  | 'color'
  | 'highlight'
  | 'heading'
  | 'textAlign'
  | 'bulletList'
  | 'orderedList'
  | 'taskList'
  | 'indent'
  | 'link'
  | 'image'
  // | 'video'
  // | 'table'
  | 'blockquote'
  | 'horizontalRule'
  | 'code'
  | 'codeBlock'
  // | 'clear'
  | 'history'

export interface GeneralOptions<T> {
  divider: boolean
  spacer: boolean
  button: ButtonView<T>
  toolbar?: boolean
  bubbleMenu?: boolean
}

export type Translate = (key: string, params?: Params, defaultValue?: string | null, route?: RouteLocationNormalizedLoaded) => CleanTranslation;

export interface ButtonViewParams<T = never> {
  editor: Editor
  extension: Extension<T>
  t: Translate
}

export interface ButtonView<T = never> {
  (options: ButtonViewParams<T>): ButtonViewReturn | ButtonViewReturn[]
}

export interface ButtonViewReturn {
  component: unknown
  componentProps: ButtonViewReturnComponentProps
  componentSlots?: ButtonViewReturnComponentSlots
}
