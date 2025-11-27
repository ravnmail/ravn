import type { Email } from '~/types/email'
import dayjs from 'dayjs'
import LocalizedFormat from 'dayjs/plugin/localizedFormat'

dayjs.extend(LocalizedFormat)

export default function useFormatting() {
  return {
    formatEmailDate(email: Email, showTimeOnlyForDaysAgo = 2, options: {
      dateFormat: string,
      timeFormat: string,
    }) {
      const { dateFormat, timeFormat } = {
        timeFormat: 'HH:mm',
        dateFormat: 'll',
        ...options
      }

      const date = dayjs(email.sent_at || email.received_at)
      if (date.isAfter(dayjs().subtract(showTimeOnlyForDaysAgo, 'day'), 'day')) {
        return date.format(timeFormat)
      }
      return date.format(dateFormat)
    }
  }
}