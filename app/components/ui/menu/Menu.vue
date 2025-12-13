<template>
  <div
    ref="menuRef"
    class="p-1 bg-popover text-popover-foreground rounded-md outline-none shadow-md flex flex-col gap-1"
    tabindex="0"
  >
    <div
      v-for="(item, index) in menuItems"
      :key="item.label"
      :class="[
        'relative flex items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors',
        item.isSelected ? 'bg-selection text-selection-foreground' : '',
        'hover:bg-selection hover:text-selection-foreground',
      ]"
      @click="handleClick(item)"
      @mouseenter="handleMouseEnter(index)"
    >
      <span>{{ item.label }}</span>
      <Icon
        v-if="item.children"
        class="ml-auto"
        name="lucide:chevron-right"
      />
      <div
        v-if="item.children && item.isSelected"
        class="absolute left-full ml-1.5 top-0 bg-popover text-popover-foreground rounded-md"
      >
        <div class="p-1 flex flex-col gap-1 min-w-32">
          <div
            v-for="(subItem, subIndex) in item.children"
            :key="subItem.label"
            :class="[
              'flex items-center rounded-sm px-2 py-1.5 text-sm whitespace-nowrap outline-none transition-colors',
              subItem.isSelected ? 'bg-selection text-selection-foreground' : '',
              'hover:bg-selection hover:text-selection-foreground',
            ]"
            @click="handleClick(subItem)"
            @mouseenter="handleSubItemMouseEnter(index, subIndex)"
          >
            <span>{{ subItem.label }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { reactive, onMounted, watch, ref } from 'vue'

interface MenuItem {
  label: string
  isHovered?: boolean
  isSelected?: boolean
  children?: MenuItem[]

  [key: string]: unknown
}

const props = defineProps<{
  items: MenuItem[]
}>()

const emit = defineEmits<{
  (e: 'itemClick', item: MenuItem): void
}>()

const menuRef = ref<HTMLElement | null>(null)
const menuItems = reactive<MenuItem[]>([])
const currentSelectedIndex = ref(0)
const currentSubMenuIndex = ref(-1)

// 初始化菜单项
const initializeMenuItems = (items: MenuItem[]) => {
  return items.map((item, index) => ({
    ...item,
    isSelected: index === 0, // 默认选中第一项
    children: item.children?.map(child => ({
      ...child,
      isSelected: false,
    })),
  }))
}

onMounted(() => {
  menuItems.splice(0, menuItems.length, ...initializeMenuItems(props.items))
  // 确保菜单容器可以接收键盘事件
  nextTick(() => {
    menuRef.value?.focus()
  })
})

watch(
  () => props.items,
  newItems => {
    menuItems.splice(0, menuItems.length, ...initializeMenuItems(newItems))
  },
  { deep: true }
)

// 处理键盘事件
const handleKeyDown = (e: KeyboardEvent) => {
  const isInSubMenu = currentSubMenuIndex.value !== -1

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      if (isInSubMenu && menuItems[currentSelectedIndex.value].children) {
        // 在子菜单中向下移动
        const children = menuItems[currentSelectedIndex.value].children!
        currentSubMenuIndex.value = (currentSubMenuIndex.value + 1) % children.length
        updateSubMenuSelection()
      } else {
        // 在主菜单中向下移动
        currentSelectedIndex.value = (currentSelectedIndex.value + 1) % menuItems.length
        currentSubMenuIndex.value = -1
        updateMainMenuSelection()
      }
      break

    case 'ArrowUp':
      e.preventDefault()
      if (isInSubMenu && menuItems[currentSelectedIndex.value].children) {
        // 在子菜单中向上移动
        const children = menuItems[currentSelectedIndex.value].children!
        currentSubMenuIndex.value = (currentSubMenuIndex.value - 1 + children.length) % children.length
        updateSubMenuSelection()
      } else {
        // 在主菜单中向上移动
        currentSelectedIndex.value = (currentSelectedIndex.value - 1 + menuItems.length) % menuItems.length
        currentSubMenuIndex.value = -1
        updateMainMenuSelection()
      }
      break

    case 'ArrowRight':
      e.preventDefault()
      if (!isInSubMenu && menuItems[currentSelectedIndex.value].children?.length) {
        // 进入子菜单
        currentSubMenuIndex.value = 0
        updateSubMenuSelection()
      }
      break

    case 'ArrowLeft':
      e.preventDefault()
      if (isInSubMenu) {
        // 退出子菜单
        currentSubMenuIndex.value = -1
        updateMainMenuSelection()
      }
      break

    case 'Enter':
      e.preventDefault()
      const currentItem = isInSubMenu
        ? menuItems[currentSelectedIndex.value].children?.[currentSubMenuIndex.value]
        : menuItems[currentSelectedIndex.value]
      if (currentItem && !currentItem.children) {
        handleClick(currentItem)
      }
      break
  }
}

defineExpose({ handleKeyDown })

// 更新主菜单选中状态
const updateMainMenuSelection = () => {
  menuItems.forEach((item, index) => {
    item.isSelected = index === currentSelectedIndex.value
    if (item.children) {
      item.children.forEach(child => {
        child.isSelected = false
      })
    }
  })
}

// 更新子菜单选中状态
const updateSubMenuSelection = () => {
  const currentItem = menuItems[currentSelectedIndex.value]
  if (currentItem.children) {
    currentItem.children.forEach((child, index) => {
      child.isSelected = index === currentSubMenuIndex.value
    })
  }
}

// 鼠标事件处理
const handleMouseEnter = (index: number) => {
  currentSelectedIndex.value = index
  currentSubMenuIndex.value = -1
  updateMainMenuSelection()
}

const handleSubItemMouseEnter = (parentIndex: number, subIndex: number) => {
  currentSelectedIndex.value = parentIndex
  currentSubMenuIndex.value = subIndex
  updateSubMenuSelection()
}

const handleClick = (item: MenuItem) => {
  if (item.children) {
    return
  }
  emit('itemClick', item)
}
</script>
