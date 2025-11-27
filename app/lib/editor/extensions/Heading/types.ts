import type { ButtonViewReturnComponentProps } from '@/types/composer'
import type { StyleValue } from 'vue'


export interface Item {
  title: string
  icon?: any
  level?: number
  isActive: NonNullable<ButtonViewReturnComponentProps['isActive']>
  action?: ButtonViewReturnComponentProps['action']
  style?: StyleValue
  shortcutKeys?: string[]
  disabled?: boolean
  divider?: boolean
  default?: boolean
}
