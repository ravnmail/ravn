import type { TaskItemOptions } from '@tiptap/extension-task-item'
import { TaskItem } from '@tiptap/extension-task-item'
import type { TaskListOptions as TiptapTaskListOptions } from '@tiptap/extension-task-list'
import { TaskList as TiptapTaskList } from '@tiptap/extension-task-list'

import ActionButton from '@/components/ActionButton.vue'

import type { GeneralOptions } from '@/types/composer'

export interface TaskListOptions extends TiptapTaskListOptions, GeneralOptions<TaskListOptions> {
  taskItem: Partial<TaskItemOptions>
}

export const TaskList = TiptapTaskList.extend<TaskListOptions>({
  addOptions() {
    return {
      ...this.parent?.(),
      HTMLAttributes: {
        class: 'task-list',
      },
      taskItem: {
        HTMLAttributes: {
          class: 'task-list-item',
        },
      },
      button: ({ editor, t }) => ({
        component: ActionButton,
        componentProps: {
          action: () => editor.chain().toggleTaskList().focus().run(),
          isActive: () => editor.isActive('taskList') || false,
          disabled: !editor.isEditable || !editor.can().toggleTaskList(),
          icon: 'list-todo',
          shortcutKeys: ['shift', 'mod', '9'],
          tooltip: t('composer.tasklist.tooltip'),
        },
      }),
    }
  },

  addExtensions() {
    return [TaskItem.configure(this.options.taskItem)]
  },
})
