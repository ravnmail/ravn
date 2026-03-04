import dayjs from 'dayjs'
import LocalizedFormat from 'dayjs/plugin/localizedFormat'

import type { Email } from '~/types/email'

dayjs.extend(LocalizedFormat)

export default function useFormatting() {
  return {
    formatNumber(num: number) {
      return Intl.NumberFormat(undefined, {
        maximumFractionDigits: 2,
        minimumFractionDigits: 2,
      }).format(num)
    },
    formatEmailDate(
      email: Email,
      showTimeOnlyForDaysAgo = 2,
      options?: {
        dateFormat?: string
        timeFormat?: string
      }
    ) {
      const dateFormat = options?.dateFormat ?? 'll'
      const timeFormat = options?.timeFormat ?? 'HH:mm'

      const date = dayjs(email.sent_at || email.received_at)
      if (date.isAfter(dayjs().subtract(showTimeOnlyForDaysAgo, 'day'), 'day')) {
        return date.format(timeFormat)
      }
      return date.format(dateFormat)
    },
  }
}

/**
 * Composable that reads regional settings and returns dayjs-based
 * date/time formatters respecting the user's configured preferences.
 */
export function useRegionalFormat() {
  const { settings } = useSettings()

  const dateFormat = computed(() => settings.value?.regional?.dateFormat ?? 'MMM D, YYYY')
  const timeFormat = computed(() => settings.value?.regional?.timeFormat ?? 'HH:mm')
  const weekdayFormat = computed(() => settings.value?.regional?.weekdayFormat ?? 'ddd')
  const startOfWeek = computed<0 | 1>(() => settings.value?.regional?.startOfWeek ?? 1)

  /**
   * Format a date string or Date as a date (no time component).
   */
  const formatDate = (date: string | Date | null | undefined): string => {
    if (!date) return ''
    return dayjs(date).format(dateFormat.value)
  }

  /**
   * Format a date string or Date as a time only (no date component).
   */
  const formatTime = (date: string | Date | null | undefined): string => {
    if (!date) return ''
    return dayjs(date).format(timeFormat.value)
  }

  /**
   * Format a date as "time" if it is within the last N days, otherwise as a date.
   */
  const formatEmailDate = (
    date: string | Date | null | undefined,
    showTimeOnlyForDaysAgo = 2
  ): string => {
    if (!date) return ''
    const d = dayjs(date)
    if (d.isAfter(dayjs().subtract(showTimeOnlyForDaysAgo, 'day'), 'day')) {
      return d.format(timeFormat.value)
    }
    return d.format(dateFormat.value)
  }

  /**
   * Format a weekday label (e.g. for calendar column headers) from a Date.
   * Uses the configured weekdayFormat token.
   */
  const formatWeekday = (date: Date): string => {
    return dayjs(date).format(weekdayFormat.value)
  }

  /**
   * Return the ordered list of weekday indices (0=Sun…6=Sat) starting from startOfWeek.
   * startOfWeek=1 → [1,2,3,4,5,6,0] (Mon–Sun)
   * startOfWeek=0 → [0,1,2,3,4,5,6] (Sun–Sat)
   */
  const weekdayOrder = computed<number[]>(() => {
    const start = startOfWeek.value
    return Array.from({ length: 7 }, (_, i) => (start + i) % 7)
  })

  /**
   * Given a Date, return the offset (in days) to go back to reach the start-of-week.
   * Compatible with both Sunday-first (0) and Monday-first (1) configurations.
   */
  const offsetToStartOfWeek = (date: Date): number => {
    const dow = date.getDay() // 0 = Sun, 1 = Mon, ..., 6 = Sat
    const start = startOfWeek.value
    return (dow - start + 7) % 7
  }

  return {
    dateFormat,
    timeFormat,
    weekdayFormat,
    startOfWeek,
    weekdayOrder,
    formatDate,
    formatTime,
    formatEmailDate,
    formatWeekday,
    offsetToStartOfWeek,
  }
}
