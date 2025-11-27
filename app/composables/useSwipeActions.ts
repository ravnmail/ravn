import { useEventListener } from '@vueuse/core'
import type { CleanTranslation } from 'nuxt-i18n-micro-types/src'

export interface SwipeAction {
  id: string
  icon: string
  label: CleanTranslation | string
  color: string
  handler?: () => void
}

export interface SwipeActionsOptions {
  leftActions?: SwipeAction[]
  rightActions?: SwipeAction[]
  threshold?: number
  fastSwipeThreshold?: number
  maxSwipeDistance?: number
  scrollSensitivity?: number // Controls how sensitive scroll-based swiping is
}

export function useSwipeActions(
  elementRef: Ref<HTMLElement | null>,
  options: SwipeActionsOptions = {}
) {
  const {
    leftActions = [],
    rightActions = [],
    threshold = 30,
    fastSwipeThreshold = 0.5, // time in seconds for fast swipe
    maxSwipeDistance = 200,
    scrollSensitivity = 1.5 // Higher values make scrolling more sensitive
  } = options

  // Detect if we're on a touch device
  const isTouchDevice = ref(false)
  const isDesktop = useMediaQuery('(pointer: fine)')
  const prefersReducedMotion = usePreferredReducedMotion()

  const swipeOffset = ref(0)
  const swipeStartX = ref(0)
  const swipeStartTime = ref(0)
  const isSwiping = ref(false)
  const activeActionSide = ref<'left' | 'right' | null>(null)
  const visibleSide = ref<'left' | 'right' | null>(null)

  // Calculate action widths
  const actionWidth = 80 // Fixed width for each action button
  const leftActionsWidth = leftActions.length * actionWidth
  const rightActionsWidth = rightActions.length * actionWidth

  // Track which action is currently active based on swipe distance
  const activeActionIndex = ref(-1)

  // Scroll handling
  const scrollAccumulator = ref(0)
  const isScrolling = ref(false)
  const scrollStartTime = ref(0)
  const scrollTimeout = ref<number | null>(null)

  const reset = () => {
    swipeOffset.value = 0
    activeActionSide.value = null
    visibleSide.value = null
    activeActionIndex.value = -1
    isSwiping.value = false
    isScrolling.value = false
    scrollAccumulator.value = 0

    if (scrollTimeout.value !== null) {
      window.clearTimeout(scrollTimeout.value)
      scrollTimeout.value = null
    }
  }

  const handleStart = (event: TouchEvent | MouseEvent) => {
    if (event instanceof TouchEvent) {
      swipeStartX.value = event.touches[0].clientX
    } else {
      swipeStartX.value = event.clientX
    }
    swipeStartTime.value = Date.now()
    isSwiping.value = true
  }

  const handleMove = (event: TouchEvent | MouseEvent) => {
    if (!isSwiping.value) return

    let currentX: number
    if (event instanceof TouchEvent) {
      currentX = event.touches[0].clientX
    } else {
      currentX = event.clientX
    }

    const diffX = currentX - swipeStartX.value

    // Determine which side we're swiping towards
    if (diffX > 0 && leftActions.length > 0) {
      activeActionSide.value = 'left'
      visibleSide.value = 'left'
      // Limit the max distance to the left actions width
      swipeOffset.value = Math.min(diffX, leftActionsWidth)
      // Calculate which action button is active
      activeActionIndex.value = Math.min(
        Math.floor(swipeOffset.value / actionWidth),
        leftActions.length - 1
      )
    } else if (diffX < 0 && rightActions.length > 0) {
      activeActionSide.value = 'right'
      visibleSide.value = 'right'
      // Limit the max distance to the right actions width (negative value)
      swipeOffset.value = Math.max(diffX, -rightActionsWidth)
      // Calculate which action button is active
      activeActionIndex.value = Math.min(
        Math.floor(Math.abs(swipeOffset.value) / actionWidth),
        rightActions.length - 1
      )
    } else {
      activeActionSide.value = null
      visibleSide.value = null
      swipeOffset.value = 0
      activeActionIndex.value = -1
    }

    // Apply resistance as we reach max distance
    if (Math.abs(diffX) > maxSwipeDistance) {
      const excess = Math.abs(diffX) - maxSwipeDistance
      const resistance = 0.3 // Higher values = more resistance
      const resistedExcess = excess * resistance

      if (diffX > 0) {
        swipeOffset.value = Math.min(maxSwipeDistance + resistedExcess, leftActionsWidth)
      } else {
        swipeOffset.value = Math.max(-maxSwipeDistance - resistedExcess, -rightActionsWidth)
      }
    }
  }

  const handleEnd = (event: TouchEvent | MouseEvent) => {
    if (!isSwiping.value) return

    const swipeDuration = (Date.now() - swipeStartTime.value) / 1000 // in seconds
    const isFastSwipe = swipeDuration < fastSwipeThreshold

    // Handle fast swipe to trigger outermost action
    if (isFastSwipe && Math.abs(swipeOffset.value) > threshold) {
      if (activeActionSide.value === 'left' && leftActions.length > 0) {
        // Trigger the outermost left action (Archive)
        leftActions[leftActions.length - 1].handler()
        reset()
        return
      } else if (activeActionSide.value === 'right' && rightActions.length > 0) {
        // Trigger the outermost right action (Delete)
        rightActions[rightActions.length - 1].handler()
        reset()
        return
      }
    }

    // For normal swipes, check if we passed the threshold
    if (Math.abs(swipeOffset.value) > threshold) {
      // Keep the swipe open to the nearest action width
      if (activeActionSide.value === 'left') {
        const snapTo = Math.ceil(swipeOffset.value / actionWidth) * actionWidth
        swipeOffset.value = Math.min(snapTo, leftActionsWidth)
      } else if (activeActionSide.value === 'right') {
        const snapTo = Math.floor(swipeOffset.value / actionWidth) * actionWidth
        swipeOffset.value = Math.max(snapTo, -rightActionsWidth)
      }
    } else {
      // Reset if below threshold
      reset()
    }

    isSwiping.value = false
  }

  // Execute an action and reset
  const executeAction = (actionIndex: number, side: 'left' | 'right') => {
    const actions = side === 'left' ? leftActions : rightActions
    if (actionIndex >= 0 && actionIndex < actions.length) {
      actions[actionIndex].handler()
    }
    reset()
  }

  // Click handler for when actions are visible
  const handleActionClick = (actionIndex: number, side: 'left' | 'right') => {
    executeAction(actionIndex, side)
  }

  // Close swipe actions when clicking outside
  const handleOutsideClick = (event: MouseEvent) => {
    if (visibleSide.value !== null && elementRef.value && !elementRef.value.contains(event.target as Node)) {
      reset()
    }
  }

  // Handle scroll wheel events for desktop
  const handleWheel = (event: WheelEvent) => {
    // Prevent default only if we're already scrolling horizontally
    // to avoid interfering with vertical page scrolling
    if (isScrolling.value || Math.abs(event.deltaX) > Math.abs(event.deltaY)) {
      event.preventDefault()
    }

    // Detect scroll intent - if more horizontal than vertical, it's likely intentional
    const isHorizontalScroll = Math.abs(event.deltaX) > Math.abs(event.deltaY)

    if (!isScrolling.value && isHorizontalScroll) {
      isScrolling.value = true
      scrollStartTime.value = Date.now()
    }

    if (isScrolling.value) {
      // Apply sensitivity multiplier to make scrolling more responsive
      scrollAccumulator.value += event.deltaX * scrollSensitivity

      // Update swipe offset based on accumulated scroll
      updateSwipeFromScroll(scrollAccumulator.value)

      // Reset scroll timeout
      if (scrollTimeout.value !== null) {
        window.clearTimeout(scrollTimeout.value)
      }

      // Set timeout to end scrolling after a brief pause
      scrollTimeout.value = window.setTimeout(() => {
        const scrollDuration = (Date.now() - scrollStartTime.value) / 1000
        const isFastScroll = scrollDuration < fastSwipeThreshold &&
                            Math.abs(scrollAccumulator.value) > threshold * 3

        if (isFastScroll) {
          // Handle fast scroll similar to fast swipe
          handleFastAction()
        } else if (Math.abs(scrollAccumulator.value) > threshold) {
          // Snap to nearest action
          snapToNearestAction()
        } else {
          // Reset if below threshold
          reset()
        }

        isScrolling.value = false
      }, 150) // Small delay to detect end of scroll gesture
    }
  }

  // Update swipe offset from scroll value
  const updateSwipeFromScroll = (scrollValue: number) => {
    const diffX = -scrollValue // Inverted because scroll right = negative deltaX

    // Use the same logic as in handleMove
    if (diffX > 0 && leftActions.length > 0) {
      activeActionSide.value = 'left'
      visibleSide.value = 'left'
      swipeOffset.value = Math.min(diffX, leftActionsWidth)
      activeActionIndex.value = Math.min(
        Math.floor(swipeOffset.value / actionWidth),
        leftActions.length - 1
      )
    } else if (diffX < 0 && rightActions.length > 0) {
      activeActionSide.value = 'right'
      visibleSide.value = 'right'
      swipeOffset.value = Math.max(diffX, -rightActionsWidth)
      activeActionIndex.value = Math.min(
        Math.floor(Math.abs(swipeOffset.value) / actionWidth),
        rightActions.length - 1
      )
    } else {
      activeActionSide.value = null
      visibleSide.value = null
      swipeOffset.value = 0
      activeActionIndex.value = -1
    }

    // Apply resistance for very large scroll values
    if (Math.abs(diffX) > maxSwipeDistance) {
      const excess = Math.abs(diffX) - maxSwipeDistance
      const resistance = 0.3
      const resistedExcess = excess * resistance

      if (diffX > 0) {
        swipeOffset.value = Math.min(maxSwipeDistance + resistedExcess, leftActionsWidth)
      } else {
        swipeOffset.value = Math.max(-maxSwipeDistance - resistedExcess, -rightActionsWidth)
      }
    }
  }

  // Handle fast action (for fast swipes/scrolls)
  const handleFastAction = () => {
    if (activeActionSide.value === 'left' && leftActions.length > 0) {
      // Trigger the outermost left action
      leftActions[leftActions.length - 1].handler()
      reset()
    } else if (activeActionSide.value === 'right' && rightActions.length > 0) {
      // Trigger the outermost right action
      rightActions[rightActions.length - 1].handler()
      reset()
    }
  }

  // Snap to nearest action width
  const snapToNearestAction = () => {
    if (activeActionSide.value === 'left') {
      const snapTo = Math.ceil(swipeOffset.value / actionWidth) * actionWidth
      swipeOffset.value = Math.min(snapTo, leftActionsWidth)
    } else if (activeActionSide.value === 'right') {
      const snapTo = Math.floor(swipeOffset.value / actionWidth) * actionWidth
      swipeOffset.value = Math.max(snapTo, -rightActionsWidth)
    }
  }

  // Detect touch capability on mount
  onMounted(() => {
    isTouchDevice.value = 'ontouchstart' in window || navigator.maxTouchPoints > 0

    if (elementRef.value) {
      // Touch events for mobile
      useEventListener(elementRef.value, 'touchstart', handleStart, { passive: true })
      useEventListener(elementRef.value, 'touchmove', handleMove, { passive: true })
      useEventListener(elementRef.value, 'touchend', handleEnd)

      // Mouse events for dragging (primarily for touch-pad users)
      useEventListener(elementRef.value, 'mousedown', handleStart)
      useEventListener(window, 'mousemove', handleMove)
      useEventListener(window, 'mouseup', handleEnd)

      // Wheel events for desktop scroll gestures
      // Only if pointer is fine (likely desktop) and it's not a touch screen
      if (isDesktop.value) {
        useEventListener(elementRef.value, 'wheel', handleWheel, { passive: false })
      }
    }

    useEventListener(window, 'click', handleOutsideClick)
  })

  onUnmounted(() => {
    reset()

    // Clean up any lingering timeouts
    if (scrollTimeout.value !== null) {
      window.clearTimeout(scrollTimeout.value)
    }
  })

  return {
    swipeOffset,
    isSwiping,
    isScrolling,
    activeActionSide,
    visibleSide,
    activeActionIndex,
    handleActionClick,
    reset,
    isDesktop,
    isTouchDevice,
    prefersReducedMotion
  }
}