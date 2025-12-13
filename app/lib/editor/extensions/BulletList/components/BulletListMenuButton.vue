<script lang="ts" setup>
import { computed } from 'vue'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '@/components/ui/tooltip'
import type { Editor } from '@tiptap/vue-3'
import ActionDropdownButtonSplit from '@/components/ActionDropdownButtonSplit.vue'
import { MenuItem } from '@/components/ui/menu'
import { SimpleTooltip } from '~/components/ui/tooltip'

interface BulletListOption {
  label: string
  value: 'disc' | 'circle' | 'square'
}

interface Props {
  editor: Editor
  disabled?: boolean
  tooltip?: string
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  tooltip: '',
})

const { t } = useI18n()

const BulletListOptions: BulletListOption[] = [
  { label: 'editor.bulletlist.disc.tooltip', value: 'disc' },
  { label: 'editor.bulletlist.circle.tooltip', value: 'circle' },
  { label: 'editor.bulletlist.square.tooltip', value: 'square' },
]

const active = computed((): 'disc' | 'circle' | 'square' => {
  if (props.editor?.isActive('bulletList')) {
    return props.editor.getAttributes('bulletList').listStyleType as 'disc' | 'circle' | 'square'
  }
  return 'disc'
})

function toggleBulletList(item: BulletListOption): void {
  if (props.editor.isActive('bulletList')) {
    if (props.editor.getAttributes('bulletList').listStyleType === item.value) {
      props.editor.chain().focus().toggleBulletList().run()
    } else {
      props.editor.chain().focus().updateAttributes('bulletList', { listStyleType: item.value }).run()
    }
  } else {
    props.editor.chain().focus().toggleBulletList().updateAttributes('bulletList', { listStyleType: item.value }).run()
  }
}
</script>

<template>
  <ActionDropdownButtonSplit
    :action="toggleBulletList"
    :disabled="disabled"
    :tooltip="tooltip"
    class="min-w-4 w-full grid grid-cols-3 gap-1"
  >
    <SimpleTooltip
      v-for="item in BulletListOptions"
      :key="item.value"
      :delay-duration="0"
      :tooltip="t(item.label)"
      side="bottom"
    >
      <MenuItem
        class="p-0!"
        @click="toggleBulletList(item)"
      >
        <div
          :class="[active === item.value ? 'bg-selection border ' : '']"
          class="h-10 flex flex-col w-12 border-input rounded-sm border"
        >
          <ul
            :style="{ listStyleType: item.value, lineHeight: 1 }"
            class="text-muted pl-3 flex-1 list-inside"
          >
            <li
              v-for="i in 3"
              :key="i"
            />
          </ul>
        </div>
      </MenuItem>
    </SimpleTooltip>
  </ActionDropdownButtonSplit>
</template>
