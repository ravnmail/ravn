<script lang="ts" setup>

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '~/components/ui/tooltip/index'
import type { HTMLAttributes } from 'vue'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import Markdown from '~/components/Markdown.vue'
import Shortcuts from '~/components/ui/kbd/Shortcuts.vue'

const props = withDefaults(defineProps<{
  tooltip?: string | CleanTranslation,
  tooltipMarkdown?: string | CleanTranslation,
  shortcutKeys?: string[],
  delayDuration?: number
  side?: 'top' | 'right' | 'bottom' | 'left'
  class?: HTMLAttributes['class']
}>(), {
  delayDuration: 300,
  shortcutKeys: [],
  side: 'top',
  class: ''
})
</script>

<template>
  <TooltipProvider :delay-duration="delayDuration">
    <Tooltip>
      <TooltipTrigger
        :class="props.class"
        type="button"
      >
        <slot/>
      </TooltipTrigger>
      <TooltipContent
        :side="side"
      >
        <p v-if="tooltip">{{ tooltip }}</p>
        <Markdown
          v-else-if="tooltipMarkdown"
          :content="tooltipMarkdown"
        />
        <Shortcuts
          v-if="shortcutKeys?.length"
          :shortcut-keys="shortcutKeys"
        />
      </TooltipContent>
    </Tooltip>
  </TooltipProvider>
</template>