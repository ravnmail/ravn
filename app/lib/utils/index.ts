import { type ClassValue, clsx } from 'clsx'

export function cn(...inputs: ClassValue[]) {
  return clsx(inputs)
}

export const isNumber = (value: unknown): value is number => typeof value === 'number'

export const isString = (value: unknown): value is string => typeof value === 'string'

export const isBoolean = (value: unknown): value is boolean => typeof value === 'boolean'

export const isFunction = (value: unknown): value is Function => typeof value === 'function'
