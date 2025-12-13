<script lang="ts" setup>
import type { MenuListProps } from './types'


const selectedCommandIndex = ref(0)
const selectedGroupIndex = ref(0)

const scrollContainer = ref<HTMLDivElement | null>(null)

const { t } = useI18n()

const activeItemRefs = ref<(HTMLButtonElement | null)[]>([])
const props = withDefaults(defineProps<MenuListProps>(), {
  items: undefined,
  command: undefined,
})

defineExpose({ onKeyDown })

watch([() => selectedCommandIndex.value, () => selectedGroupIndex.value], async () => {
  if (!scrollContainer.value) return
  await nextTick()
  const activeItemIndex = selectedGroupIndex.value * 1000 + selectedCommandIndex.value

  const activeItem = activeItemRefs.value[activeItemIndex]
  if (activeItem) {
    activeItem.scrollIntoView({
      behavior: 'smooth',
      block: 'nearest',
    })
  }
})

function onKeyDown({ event }) {
  if (event.key === 'ArrowUp') {
    upHandler()
    return true
  }

  if (event.key === 'ArrowDown') {
    downHandler()
    return true
  }

  if (event.key === 'Enter') {
    enterHandler()
    return true
  }

  return false
}

function upHandler() {
  if (!props.items.length) {
    return false
  }
  let newCommandIndex = selectedCommandIndex.value - 1
  let newGroupIndex = selectedGroupIndex.value

  if (newCommandIndex < 0) {
    newGroupIndex = selectedGroupIndex.value - 1
    newCommandIndex = props.items[newGroupIndex]?.commands.length - 1 || 0
  }

  if (newGroupIndex < 0) {
    newGroupIndex = props.items.length - 1
    newCommandIndex = props.items[newGroupIndex].commands.length - 1
  }

  selectedCommandIndex.value = newCommandIndex
  selectedGroupIndex.value = newGroupIndex
}

function downHandler() {
  if (!props.items.length) {
    return false
  }
  const commands = props.items[selectedGroupIndex.value].commands
  let newCommandIndex = selectedCommandIndex.value + 1
  let newGroupIndex = selectedGroupIndex.value

  if (commands.length - 1 < newCommandIndex) {
    newCommandIndex = 0
    newGroupIndex = selectedGroupIndex.value + 1
  }
  if (props.items.length - 1 < newGroupIndex) {
    newGroupIndex = 0
  }
  selectedCommandIndex.value = newCommandIndex
  selectedGroupIndex.value = newGroupIndex
}

function enterHandler() {
  if (!props.items.length || selectedGroupIndex.value === -1 || selectedCommandIndex.value === -1) {
    return false
  }

  selectItem(selectedGroupIndex.value, selectedCommandIndex.value)
}

function selectItem(groupIndex: number, commandIndex: number) {
  const command = props.items[groupIndex].commands[commandIndex]
  props.command(command)
}

function createCommandClickHandler(groupIndex: number, commandIndex: number) {
  selectItem(groupIndex, commandIndex)
}

function setActiveItemRef(groupIndex: number, commandIndex: number, el: unknown) {
  activeItemRefs.value[groupIndex * 1000 + commandIndex] = el
}
</script>
<template>
  <div
    ref="scrollContainer"
    class="bg-popover border-popover-border border p-1 rounded-md shadow-md max-h-80 overflow-y-auto"
  >
    <div
      v-if="items?.length"
      class="grid grid-cols-1 gap-px min-w-64"
    >
      <template
        v-for="(group, groupIndex) in items"
        :key="group.title"
      >
        <div
          className="text-muted text-xs col-[1/-1] mx-2 mt-2 font-semibold tracking-wider select-none uppercase first:mt-0.5"
        >
          {{ group.title }}
        </div>
        <button
          v-for="(command, commandIndex) in group.commands"
          :key="commandIndex"
          :ref="el => setActiveItemRef(groupIndex, commandIndex, el)"
          :class="[
            selectedGroupIndex === groupIndex && selectedCommandIndex === commandIndex
              ? 'bg-selection text-selection-foreground'
              : 'hover:bg-selection hover:text-selection-foreground',
          ]"
          class="flex items-center gap-2 px-2 py-1.5 text-sm text-left w-full rounded-sm outline-none transition-colors"
          @click="createCommandClickHandler(groupIndex, commandIndex)"
        >
          <Icon
            v-if="command.iconName"
            :name="`lucide:${command.iconName}`"
            :size="16"
          />
          <span class="grow">{{ command.label }}</span>
          <span class="text-muted">{{ command.shortcut }}</span>
        </button>
      </template>
    </div>
    <div
      v-else
      class="p-3"
    >
      <span class="text-xs text-muted">{{ t('composer.slash.empty') }}</span>
    </div>
  </div>
</template>
