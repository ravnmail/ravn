<script lang="ts" setup>
import dayjs from 'dayjs'

import CalendarEmailItem from '~/components/Ravn/CalendarEmailItem.vue'
import ConversationViewer from '~/components/Ravn/ConversationViewer.vue'
import MailContextMenu from '~/components/Ravn/MailContextMenu.vue'
import { useSelectedConversation } from '~/components/Ravn/view/useSelectedConversation'
import { Button } from '~/components/ui/button'
import { UnobstrusiveSheetContent } from '~/components/ui/sheet'
import { ToggleGroup, ToggleGroupItem } from '~/components/ui/toggle-group'
import { useMultiSelect } from '~/composables/useDragAndDrop'
import { useRegionalFormat } from '~/composables/useFormatting'
import type { EmailListItem } from '~/types/email'
import type { CalendarMode, CalendarViewConfig, View } from '~/types/view'

const props = defineProps<{
  view: View
}>()

const { t } = useI18n()
const { fetchForCalendar, archive, trash, move, updateRead, addLabelToEmail, setRemindAt } =
  useEmails()
const { updateView } = useViews()
const { startOfWeek, weekdayOrder, formatWeekday, offsetToStartOfWeek } = useRegionalFormat()
const multiSelect = useMultiSelect<EmailListItem>()

const config = computed(() => props.view.config as CalendarViewConfig)

const mode = ref<CalendarMode>(config.value.mode ?? 'month')

let persistTimer: ReturnType<typeof setTimeout> | null = null
const persistMode = (newMode: CalendarMode) => {
  if (persistTimer) clearTimeout(persistTimer)
  persistTimer = setTimeout(async () => {
    try {
      await updateView({
        id: props.view.id,
        name: props.view.name,
        icon: props.view.icon,
        color: props.view.color,
        view_type: props.view.view_type,
        config: {
          ...config.value,
          mode: newMode,
        },
        folders: props.view.folders,
        sort_order: props.view.sort_order,
      })
    } catch (e) {
      console.error('[CalendarBoard] Failed to persist mode:', e)
    }
  }, 600)
}

const setMode = (newMode: CalendarMode) => {
  mode.value = newMode
  persistMode(newMode)
}

const today = dayjs().startOf('day')

// anchorDate is always stored as a plain Date for reactivity simplicity
const anchorDate = ref<Date>(today.toDate())

const goToToday = () => {
  anchorDate.value = today.toDate()
}

const goBackward = () => {
  const d = dayjs(anchorDate.value)
  anchorDate.value = (
    mode.value === 'month' ? d.subtract(1, 'month') : d.subtract(1, 'week')
  ).toDate()
}

const goForward = () => {
  const d = dayjs(anchorDate.value)
  anchorDate.value = (mode.value === 'month' ? d.add(1, 'month') : d.add(1, 'week')).toDate()
}

// When switching modes, snap anchorDate to a sensible starting point
watch(mode, (newMode) => {
  const d = dayjs(anchorDate.value)
  if (newMode === 'week') {
    // Snap to start of the week that contains anchorDate
    anchorDate.value = d.subtract(offsetToStartOfWeek(d.toDate()), 'day').startOf('day').toDate()
  } else {
    // Snap to first of the month
    anchorDate.value = d.startOf('month').toDate()
  }
})

// Also re-snap the week start when startOfWeek setting changes
watch(startOfWeek, () => {
  if (mode.value === 'week') {
    const d = dayjs(anchorDate.value)
    anchorDate.value = d.subtract(offsetToStartOfWeek(d.toDate()), 'day').startOf('day').toDate()
  }
})

// ─── Weekday column header labels ────────────────────────────────────────────

// Build ordered weekday label array that respects startOfWeek and weekdayFormat
const weekDayLabels = computed<string[]>(() => {
  return weekdayOrder.value.map((dow) => {
    // dayjs week starts on Sunday (0); use a reference date on that day-of-week
    // Reference: 2025-01-05 (Sun) + offset = desired weekday
    const referenceDate = dayjs('2025-01-05').add(dow, 'day')
    return formatWeekday(referenceDate.toDate())
  })
})

// ─── Period label ─────────────────────────────────────────────────────────────

const periodLabel = computed(() => {
  if (mode.value === 'month') {
    return dayjs(anchorDate.value).format('MMMM YYYY')
  }

  const days = weekCalendarDays.value
  if (!days.length) return ''
  const first = dayjs(days[0]!.date)
  const last = dayjs(days[days.length - 1]!.date)

  const sameMonth = first.month() === last.month()
  const sameYear = first.year() === last.year()

  const fmtStart = first.format(sameYear ? 'D MMM' : 'D MMM YYYY')
  const fmtEnd = last.format(sameMonth ? 'D' : sameYear ? 'D MMM' : 'D MMM YYYY')
  return `${fmtStart} – ${fmtEnd}`
})

// ─── Calendar grid data ───────────────────────────────────────────────────────

interface CalendarDay {
  date: Date
  dateKey: string
  isCurrentPeriod: boolean
  isToday: boolean
}

const todayKey = today.format('YYYY-MM-DD')

function toDateKey(d: Date | dayjs.Dayjs): string {
  return dayjs(d).format('YYYY-MM-DD')
}

const monthCalendarDays = computed<CalendarDay[]>(() => {
  const anchor = dayjs(anchorDate.value)
  const year = anchor.year()
  const month = anchor.month()

  const firstDay = dayjs(new Date(year, month, 1))
  const lastDay = dayjs(new Date(year, month + 1, 0))

  const start = startOfWeek.value // 0 = Sun, 1 = Mon

  // Offset from grid column 0 to the first day of the month
  const firstDow = firstDay.day() // 0=Sun … 6=Sat
  const startOffset = (firstDow - start + 7) % 7

  // Offset from last day of month to end of the final grid row
  const lastDow = lastDay.day()
  const endOffset = 6 - ((lastDow - start + 7) % 7)

  const days: CalendarDay[] = []

  for (let i = startOffset - 1; i >= 0; i--) {
    const d = firstDay.subtract(i + 1, 'day')
    days.push({ date: d.toDate(), dateKey: toDateKey(d), isCurrentPeriod: false, isToday: false })
  }

  for (let d = 0; d < lastDay.date(); d++) {
    const date = dayjs(new Date(year, month, d + 1))
    const dateKey = toDateKey(date)
    days.push({
      date: date.toDate(),
      dateKey,
      isCurrentPeriod: true,
      isToday: dateKey === todayKey,
    })
  }

  for (let i = 1; i <= endOffset; i++) {
    const d = lastDay.add(i, 'day')
    days.push({ date: d.toDate(), dateKey: toDateKey(d), isCurrentPeriod: false, isToday: false })
  }

  return days
})

const monthRowCount = computed(() => Math.ceil(monthCalendarDays.value.length / 7))

const weekCalendarDays = computed<CalendarDay[]>(() => {
  const base = dayjs(anchorDate.value)
  const offset = offsetToStartOfWeek(base.toDate())
  const weekStart = base.subtract(offset, 'day').startOf('day')

  return Array.from({ length: 7 }, (_, i) => {
    const d = weekStart.add(i, 'day')
    const dateKey = toDateKey(d)
    return { date: d.toDate(), dateKey, isCurrentPeriod: true, isToday: dateKey === todayKey }
  })
})

const activeDays = computed(() =>
  mode.value === 'month' ? monthCalendarDays.value : weekCalendarDays.value
)

// ─── Email data fetching ──────────────────────────────────────────────────────

const emailsByDate = ref<Record<string, EmailListItem[]>>({})
const isLoadingEmails = ref(false)

const loadEmails = async () => {
  isLoadingEmails.value = true
  try {
    const days = activeDays.value
    if (!days.length) return

    const start = dayjs(days[0]!.date).startOf('day')
    const end = dayjs(days[days.length - 1]!.date).endOf('day')

    const emails = await fetchForCalendar(
      config.value.folder_ids || [],
      config.value.date_field || 'remind_at',
      start.toISOString(),
      end.toISOString()
    )

    const grouped: Record<string, EmailListItem[]> = {}
    for (const email of emails) {
      const rawDate =
        config.value.date_field === 'sent_at'
          ? email.sent_at
          : config.value.date_field === 'remind_at'
            ? email.remind_at
            : email.received_at
      if (!rawDate) continue
      const key = toDateKey(dayjs(rawDate))
      if (!grouped[key]) grouped[key] = []
      grouped[key].push(email)
    }
    emailsByDate.value = grouped
  } finally {
    isLoadingEmails.value = false
  }
}

watch([anchorDate, mode, startOfWeek], () => loadEmails(), { immediate: true })

const getAllEmails = () => {
  return Object.values(emailsByDate.value).flat()
}

const getSelectedEmails = () => {
  const selectedIds = multiSelect.selectedIds.value
  if (selectedIds.length === 0) return []

  const selectedIdSet = new Set(selectedIds)
  return getAllEmails().filter((email) => selectedIdSet.has(email.id))
}

const selectedEmailIds = computed(() => {
  const ids = new Set(multiSelect.selectedIds.value)

  if (selectedConversationId.value) {
    const primarySelectedEmail = getAllEmails().find(
      (email) => email.conversation_id === selectedConversationId.value
    )
    if (primarySelectedEmail) {
      ids.add(primarySelectedEmail.id)
    }
  }

  return Array.from(ids)
})

const activeContextEmail = computed(() => {
  if (selectedEmailIds.value.length !== 1) return null
  return getAllEmails().find((email) => email.id === selectedEmailIds.value[0]) ?? null
})

const handleSelect = (email: EmailListItem, event?: MouseEvent) => {
  if (event?.metaKey || event?.ctrlKey || event?.shiftKey) {
    if (selectedConversationId.value && !multiSelect.selectedIds.value.includes(email.id)) {
      const primarySelectedEmail = getAllEmails().find(
        (candidate) => candidate.conversation_id === selectedConversationId.value
      )
      if (
        primarySelectedEmail &&
        !multiSelect.selectedIds.value.includes(primarySelectedEmail.id)
      ) {
        multiSelect.toggleSelect(primarySelectedEmail)
      }
    }

    multiSelect.toggleSelect(email, event)
    selectConversation(email.conversation_id)
  } else {
    multiSelect.clearSelection()
    selectConversation(email.conversation_id)
  }
}

const ensureContextSelection = (email: EmailListItem) => {
  if (!selectedEmailIds.value.includes(email.id)) {
    multiSelect.clearSelection()
    multiSelect.toggleSelect(email)
    selectConversation(email.conversation_id)
  }
}

const executeAction = async (actionId: string, arg?: unknown) => {
  const selectedIdSet = new Set(selectedEmailIds.value)
  const emails = getAllEmails().filter((email) => selectedIdSet.has(email.id))
  if (emails.length === 0) return

  switch (actionId) {
    case 'archiveEmail':
      await Promise.all(emails.map((email) => archive(email.id)))
      break
    case 'deleteEmail':
      await Promise.all(emails.map((email) => trash(email.id)))
      break
    case 'moveEmail':
      await Promise.all(emails.map((email) => move(email.id, arg as string)))
      break
    case 'markRead':
      await Promise.all(emails.map((email) => updateRead(email.id, true)))
      break
    case 'markUnread':
      await Promise.all(emails.map((email) => updateRead(email.id, false)))
      break
    case 'assignLabel':
      await Promise.all(
        emails.map((email) => addLabelToEmail({ email_id: email.id, label_id: arg as string }))
      )
      break
    case 'setRemindAt':
      await Promise.all(
        emails.map((email) => setRemindAt(email.id, (arg as string | null) ?? null))
      )
      break
  }

  const mutating = ['archiveEmail', 'deleteEmail', 'moveEmail', 'setRemindAt']
  if (mutating.includes(actionId)) {
    await loadEmails()
  }
}

const handleEmailAction = async (actionId: string) => {
  const mutating = ['archiveEmail', 'deleteEmail', 'moveEmail', 'setRemindAt']
  if (mutating.includes(actionId)) {
    await loadEmails()
  }
}

// ─── Conversation sheet ───────────────────────────────────────────────────────

const { selectedConversationId, selectConversation, clearSelectedConversation } =
  useSelectedConversation()

const onSheetClose = () => {
  clearSelectedConversation()
}

const dateFieldLabel = computed(() => {
  switch (config.value.date_field) {
    case 'sent_at':
      return t('components.calendar.dateField.sentAt')
    case 'received_at':
      return t('components.calendar.dateField.receivedAt')
    default:
      return t('components.calendar.dateField.remindAt')
  }
})

// ─── Day number label ─────────────────────────────────────────────────────────

const formatDayNumber = (date: Date): string => dayjs(date).format('D')

const formatDayMonthShort = (date: Date): string => dayjs(date).format('MMM')
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden">
    <!-- Toolbar -->
    <div class="flex shrink-0 items-center gap-1 border-b px-3 py-2">
      <slot />

      <div class="relative z-1 ml-auto flex items-center gap-1">
        <ToggleGroup
          type="single"
          :model-value="mode"
          @update:model-value="(v) => v && setMode(v as CalendarMode)"
        >
          <ToggleGroupItem
            value="month"
            size="sm"
            class="h-7 gap-1.5 px-2.5 data-[state=on]:bg-background data-[state=on]:shadow-sm"
          >
            <Icon name="lucide:calendar-range" />
            {{ t('components.calendar.mode.month') }}
          </ToggleGroupItem>
          <ToggleGroupItem
            value="week"
            size="sm"
            class="h-7 gap-1.5 px-2.5 data-[state=on]:bg-background data-[state=on]:shadow-sm"
          >
            <Icon name="lucide:calendar-days" />
            {{ t('components.calendar.mode.week') }}
          </ToggleGroupItem>
        </ToggleGroup>

        <div class="mx-1 h-5 w-px bg-border" />

        <Button
          size="sm"
          variant="outline"
          @click="goToToday"
        >
          {{ t('components.calendar.today') }}
        </Button>

        <Button
          size="icon"
          variant="ghost"
          @click="goBackward"
        >
          <Icon name="lucide:chevron-left" />
        </Button>

        <span class="min-w-44 text-center text-sm font-semibold tabular-nums">
          {{ periodLabel }}
        </span>

        <Button
          size="icon"
          variant="ghost"
          @click="goForward"
        >
          <Icon name="lucide:chevron-right" />
        </Button>

        <Button
          size="icon"
          variant="ghost"
          @click="loadEmails"
        >
          <Icon
            name="lucide:refresh-cw"
            :class="{ 'animate-spin': isLoadingEmails }"
          />
        </Button>
      </div>
    </div>

    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div class="grid shrink-0 grid-cols-7 border-b">
        <div
          v-for="label in weekDayLabels"
          :key="label"
          class="text-muted-foreground border-r py-1.5 text-center text-xs font-medium tracking-wide uppercase last:border-r-0"
        >
          {{ label }}
        </div>
      </div>
      <div
        v-if="mode === 'month'"
        :style="{ gridTemplateRows: `repeat(${monthRowCount}, minmax(0, 1fr))` }"
        class="grid min-h-0 flex-1 grid-cols-7 overflow-hidden"
      >
        <div
          v-for="day in monthCalendarDays"
          :key="day.dateKey"
          :class="[
            'group/cell flex flex-col overflow-hidden border-r border-b last:border-r-0',
            day.isCurrentPeriod ? 'bg-background' : 'bg-surface',
          ]"
        >
          <div
            :class="[
              'flex shrink-0 items-center justify-between px-2 pt-1 pb-0.5',
              !day.isCurrentPeriod && 'opacity-40',
            ]"
          >
            <span
              :class="[
                'inline-flex h-6 w-6 items-center justify-center rounded-full text-xs font-medium',
                day.isToday ? 'bg-primary text-primary-foreground' : 'text-foreground',
              ]"
            >
              {{ formatDayNumber(day.date) }}
            </span>
            <span
              v-if="(emailsByDate[day.dateKey] || []).length > 0"
              class="text-muted-foreground text-[10px] tabular-nums"
            >
              {{ (emailsByDate[day.dateKey] || []).length }}
            </span>
          </div>

          <div class="min-h-0 flex-1 overflow-y-auto px-1 pb-1">
            <MailContextMenu
              :active-email="activeContextEmail"
              :selected-email-ids="selectedEmailIds"
              :on-execute-action="executeAction"
            >
              <div class="flex flex-col gap-1">
                <CalendarEmailItem
                  v-for="email in emailsByDate[day.dateKey] || []"
                  :key="email.id"
                  :email="email"
                  :is-selected="
                    multiSelect.isSelected(email.id).value ||
                    email.conversation_id === selectedConversationId
                  "
                  :selected-ids="selectedEmailIds"
                  @click="handleSelect(email, $event)"
                  @action="handleEmailAction"
                  @contextmenu.capture="ensureContextSelection(email)"
                />
              </div>
            </MailContextMenu>
            <div
              v-if="day.isToday && !(emailsByDate[day.dateKey] || []).length"
              class="text-muted-foreground/40 flex min-h-6 items-center justify-center text-[10px]"
            >
              {{ t('components.calendar.noItems') }}
            </div>
          </div>
        </div>
      </div>
      <div
        v-else
        class="grid min-h-0 flex-1 grid-cols-7 overflow-hidden"
      >
        <div
          v-for="day in weekCalendarDays"
          :key="day.dateKey"
          :class="[
            'group/cell flex flex-col overflow-hidden border-r border-b last:border-r-0',
            day.isToday ? 'bg-background' : 'bg-surface',
          ]"
        >
          <div class="flex shrink-0 flex-col items-center gap-1 px-2 py-2">
            <span
              :class="[
                'inline-flex h-7 w-7 items-center justify-center rounded-full text-sm font-semibold',
                day.isToday ? 'bg-primary text-primary-foreground' : 'bg-surface text-foreground',
              ]"
            >
              {{ formatDayNumber(day.date) }}
            </span>
            <span
              :class="[
                'text-xs font-medium tracking-wide uppercase',
                day.isToday ? 'text-primary' : 'text-muted',
              ]"
            >
              {{ formatDayMonthShort(day.date) }}
            </span>
          </div>

          <div class="min-h-0 flex-1 overflow-y-auto p-1.5">
            <MailContextMenu
              :active-email="activeContextEmail"
              :selected-email-ids="selectedEmailIds"
              :on-execute-action="executeAction"
            >
              <div class="flex flex-col gap-1.5">
                <CalendarEmailItem
                  v-for="email in emailsByDate[day.dateKey] || []"
                  :key="email.id"
                  :email="email"
                  :is-selected="
                    multiSelect.isSelected(email.id).value ||
                    email.conversation_id === selectedConversationId
                  "
                  :selected-ids="selectedEmailIds"
                  @click="handleSelect(email, $event)"
                  @action="handleEmailAction"
                  @contextmenu.capture="ensureContextSelection(email)"
                />
              </div>
            </MailContextMenu>
          </div>
        </div>
      </div>
    </div>

    <UnobstrusiveSheetContent
      v-if="selectedConversationId"
      @close="onSheetClose()"
    >
      <ConversationViewer
        :conversation-id="selectedConversationId"
        title-class="pl-8"
      />
    </UnobstrusiveSheetContent>
  </div>
</template>
