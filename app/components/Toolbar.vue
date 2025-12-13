<script lang="ts" setup>
import { computed, unref } from 'vue'
import type { Editor } from '@tiptap/core'
import type { ButtonViewReturn } from '@/types/composer'
import { Separator } from '@/components/ui/separator'
import { isFunction } from '@/lib/utils'
import { ToolbarRoot } from 'reka-ui'

interface Menu {
  button: ButtonViewReturn
  divider: boolean
  spacer: boolean
}

interface Props {
  editor: Editor
  disabled?: boolean
}

const { t } = useI18n()

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const items = computed(() => {
  const extensions = [...props.editor.extensionManager.extensions]
  const sortExtensions = extensions.sort((arr, acc) => {
    const a = arr.options?.sort ?? -1
    const b = acc.options?.sort ?? -1
    return a - b
  })

  let menus: Menu[] = []

  for (const extension of sortExtensions) {
    if (!extension.options) continue
    const { button, divider = false, spacer = false, toolbar = true } = extension.options
    if (!button || !isFunction(button) || !toolbar) continue

    const _button: ButtonViewReturn = button({
      editor: props.editor,
      extension,
      t: unref(t),
    })

    if (Array.isArray(_button)) {
      const menu: Menu[] = _button.map((k, i) => ({
        button: k,
        divider: i === _button.length - 1 ? divider : false,
        spacer: i === 0 ? spacer : false,
      }))
      menus = [...menus, ...menu]
      continue
    }

    menus.push({ button: _button, divider, spacer })
  }
  return menus
})
</script>

<template>
  <ToolbarRoot
    v-if="items.length"
    class="sticky top-0 h-auto bg-toolbar-background z-10 overflow-visible"
  >
    <div class="flex flex-nowrap overflow-x-auto sm:flex-wrap items-center py-0.5">
      <template
        v-for="(item, key) in items"
        :key="key"
      >
        <component
          :is="item.button.component"
          :disabled="disabled || item.button.componentProps?.disabled"
          :editor="editor"
          v-bind="item.button.componentProps"
        >
          <template
            v-for="(element, slotName, i) in item.button.componentSlots"
            :key="i"
            #[`${slotName}`]="values"
          >
            <component
              :is="element"
              v-bind="values?.props"
            />
          </template>
        </component>
        <Separator
          v-if="item.divider"
          class="h-auto mx-2"
          orientation="vertical"
        />
      </template>
    </div>
  </ToolbarRoot>
</template>
