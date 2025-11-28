<script lang="ts" setup>
import { MenuCheckboxItem } from '@/components/ui/menu'
import type { Editor } from '@tiptap/vue-3'
import ActionDropdownButton from '@/components/ActionDropdownButton.vue'

interface ContentTypeMenu {
  name: string
  label: string
  iconName: string
  action?: (value?: unknown) => void
  isActive: () => boolean
}

interface Props {
  editor: Editor
  disabled?: boolean
  color?: string
  maxHeight?: string | number
  icon?: string
  tooltip?: string
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  color: undefined,
  maxHeight: undefined,
  icon: undefined,
  tooltip: '',
  items: () => [],
})
const { t } = useI18n()

const menus = computed<ContentTypeMenu[]>(() => {
  return [
    {
      name: 'paragraph',
      label: t('composer.paragraph'),
      iconName: 'type',
      isActive: () =>
        props.editor.isActive('paragraph') &&
        !props.editor.isActive('orderedList') &&
        !props.editor.isActive('bulletList') &&
        !props.editor.isActive('taskList'),
      action: () => props.editor.chain().focus().clearNodes().focus().run(),
    },
    {
      name: 'heading1',
      label: t('composer.headings.h1'),
      isActive: () => props.editor.isActive('heading', { level: 1 }),
      iconName: 'heading-1',
      action: () => props.editor.chain().focus().clearNodes().toggleHeading({ level: 1 }).focus().run(),
    },
    {
      name: 'heading2',
      label: t('composer.headings.h2'),
      isActive: () => props.editor.isActive('heading', { level: 2 }),
      iconName: 'heading-2',
      action: () => props.editor.chain().focus().clearNodes().toggleHeading({ level: 2 }).focus().run(),
    },
    {
      name: 'heading3',
      label: t('composer.headings.h3'),
      isActive: () => props.editor.isActive('heading', { level: 3 }),
      iconName: 'heading-3',
      action: () => props.editor.chain().focus().clearNodes().toggleHeading({ level: 3 }).focus().run(),
    },
    {
      name: 'bulletList',
      label: t('composer.bulletlist.tooltip'),
      isActive: () => props.editor.isActive('bulletList'),
      iconName: 'list',
      action: () => props.editor.chain().focus().clearNodes().toggleBulletList().focus().run(),
    },
    {
      name: 'numberedList',
      label: t('composer.orderedlist.tooltip'),
      isActive: () => props.editor.isActive('orderedList'),
      iconName: 'list-ordered',
      action: () => props.editor.chain().focus().clearNodes().toggleOrderedList().focus().run(),
    },
    {
      name: 'taskList',
      label: t('composer.tasklist.tooltip'),
      isActive: () => props.editor.isActive('taskList'),
      iconName: 'list-todo',
      action: () => props.editor.chain().focus().clearNodes().toggleTaskList().focus().run(),
    },
    {
      name: 'blockquote',
      label: t('composer.blockquote.tooltip'),
      isActive: () => props.editor.isActive('blockquote'),
      iconName: 'text-quote',
      action: () => props.editor.chain().focus().clearNodes().toggleBlockquote().focus().run(),
    },
    {
      name: 'codeBlock',
      label: t('composer.codeblock.tooltip'),
      isActive: () => props.editor.isActive('codeBlock'),
      iconName: 'code-2',
      action: () => props.editor.chain().focus().clearNodes().setCodeBlock().focus().run(),
    },
  ]
})
const activeItem = computed(() => {
  return (
    menus.value.filter(item => item.isActive()).pop() ?? {
      label: t('composer.modify'),
    }
  )
})
</script>

<template>
  <ActionDropdownButton
    :side-offset="5"
    :title="activeItem?.label"
  >
    <MenuCheckboxItem
      v-for="(item, index) in menus"
      :key="index"
      :model-value="item.isActive?.() || false"
      @click="item.action"
    >
      <div class="flex items-center gap-2 px-2">
        <Icon
          :name="`lucide:${item.iconName}`"
          class="h3 w-3"
        />
        <span> {{ item.label }}</span>
      </div>
    </MenuCheckboxItem>
  </ActionDropdownButton>
</template>
