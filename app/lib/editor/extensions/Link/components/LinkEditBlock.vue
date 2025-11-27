<script lang="ts" setup>
import { reactive, watchEffect, ref } from 'vue'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import type { Editor } from '@tiptap/vue-3'
import { useFocus } from '@vueuse/core'

interface Props {
  editor: Editor
}

const props = withDefaults(defineProps<Props>(), {})
const emits = defineEmits(['onSetLink', 'onClickOutside'])

const { t } = useI18n()

let form = reactive({
  text: '',
  link: '',
})
const inputRef = ref<HTMLInputElement | null>(null)
const { focused } = useFocus(inputRef)
const openInNewTab = ref<boolean>(false)
const target = ref(null)
onClickOutside(target, event => emits('onClickOutside', event))

watchEffect(() => {
  const { href: link, target } = props.editor.getAttributes('link')
  const { from, to } = props.editor.state.selection
  const text = props.editor.state.doc.textBetween(from, to, ' ')
  form = {
    link,
    text,
  }
  openInNewTab.value = target === '_blank' ? true : false
})

function handleSubmit() {
  emits('onSetLink', form.link, form.text, openInNewTab.value)
}

onMounted(() => {
  focused.value = true
})
</script>

<template>
  <div
    ref="target"
    class="p-2 bg-white rounded-lg dark:bg-black shadow-sm border border-neutral-200 dark:border-neutral-800"
  >
    <form
      class="flex flex-col gap-2"
      @submit.prevent="handleSubmit"
    >
      <Label> {{ t('composer.link.dialog.text') }} </Label>
      <div class="flex w-full max-w-sm items-center gap-1.5">
        <div class="relative w-full max-w-sm items-center">
          <Input
            v-model="form.text"
            class="w-80"
            required
            type="text"
          />
        </div>
      </div>
      <Label>{{ t('composer.link.dialog.link') }}</Label>
      <div class="flex w-full max-w-sm items-center gap-1.5">
        <div class="relative w-full max-w-sm items-center">
          <Input
            ref="inputRef"
            v-model="form.link"
            :placeholer="t('composer.link.dialog.linkPlaceholder')"
            class="pl-8"
            required
            type="url"
          />
          <span class="absolute start-0 inset-y-0 flex items-center justify-center px-2">
            <Icon
              class="size-5 text-muted-foreground"
              name="lucide:link"
            />
          </span>
        </div>
      </div>
      <Button
        class="mt-2 self-end"
        type="submit"
      >{{ t('composer.link.dialog.button.apply') }}
      </Button>
    </form>
  </div>
</template>
