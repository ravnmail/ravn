<script lang="ts" setup>
import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import {
  DropdownMenuItem,
  type DropdownMenuItemEmits,
  type DropdownMenuItemProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

import { cn } from '@/lib/utils'
import DropdownMenuShortcut from '~/components/ui/dropdown-menu/DropdownMenuShortcut.vue'

const props = defineProps<
  DropdownMenuItemProps & {
    class?: HTMLAttributes['class']
    icon?: string
    iconColor?: string
    shortcut?: string[] | string
    label?: string | CleanTranslation
    inset?: boolean
  }
>()

const emits = defineEmits<DropdownMenuItemEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <DropdownMenuItem
    :class="
      cn(
        'relative flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm font-medium transition-colors outline-none focus:bg-selection focus:text-selection-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&>svg]:size-4 [&>svg]:shrink-0',
        inset && 'pl-7',
        props.class
      )
    "
    v-bind="forwardedProps"
  >
    <Icon
      v-if="icon"
      :name="icon"
      :size="16"
      :style="{ color: iconColor }"
    />
    <span
      v-if="label"
      class="flex-1"
      >{{ label }}</span
    >
    <DropdownMenuShortcut v-if="shortcut">
      {{ Array.isArray(shortcut) ? shortcut.join(', ') : shortcut }}
    </DropdownMenuShortcut>
  </DropdownMenuItem>
</template>
