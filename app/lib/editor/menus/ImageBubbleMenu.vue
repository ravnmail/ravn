<script lang="ts" setup>
import type { Editor } from '@tiptap/vue-3'
import { BubbleMenu, isActive } from '@tiptap/vue-3'
import { sticky } from 'tippy.js'
import { getRenderContainer } from '@/utils/getRenderContainer'
import { deleteSelection } from '@tiptap/pm/commands'

interface Props {
  editor: Editor
  disabled?: boolean
}
type ImageAlignments = 'left' | 'center' | 'right'

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})
const { t } = useI18n()
const imagePercent = ref('100')
const width = ref()
const height = ref()
const aspectRatio = ref()
const imageAlign: ImageAlignments[] = ['left', 'center', 'right']
const alignIconMap = {
  left: 'AlignLeft',
  center: 'AlignCenter',
  right: 'AlignRight',
}

function updateImageSize(event?: Event) {
  event?.preventDefault()
  const imageAttrs = props.editor.getAttributes('image')
  if (imageAttrs.src) {
    props.editor
      .chain()
      .focus(undefined, { scrollIntoView: false })
      .updateImage({
        width: width.value ? `${width.value}px` : null,
      })
      .run()
  }
}
function changeImagePercent(event?: any) {
  event?.preventDefault()
  const percent = Math.max(0, Math.min(100, parseInt(imagePercent.value)))
  props.editor
    .chain()
    .focus(undefined, { scrollIntoView: false })
    .updateImage({ width: `${percent}%` })
    .run()
}
const shouldShow = ({ editor }) => isActive(editor.view.state, 'image')

const getReferenceClientRect = computed(() => {
  const renderContainer = getRenderContainer(props.editor, 'node-image')
  return renderContainer?.getBoundingClientRect() || new DOMRect(-1000, -1000, 0, 0)
})

function setImageAlign(align: ImageAlignments) {
  props.editor.chain().focus().setTextAlign(align).run()
}
watch(imagePercent, () => {
  if (imagePercent.value) {
    changeImagePercent()
  }
})

watch(
  () => props.editor.getAttributes('image'),
  image => {
    if (image) {
      width.value = Math.round(parseFloat(image.originWidth))
      height.value = Math.round(parseFloat(image.originHeight))
      aspectRatio.value = image.originWidth / image.originHeight
    }
  }
)
function updateWidthFromHeight() {
  if (height.value && aspectRatio.value) {
    width.value = Math.max(30, Math.round(height.value * aspectRatio.value))
  } else {
    width.value = null
  }
}
function updateHeightFromWidth() {
  if (width.value && aspectRatio.value) {
    height.value = Math.max(20, Math.round(width.value / aspectRatio.value))
  } else {
    height.value = null
  }
}
function handleSetImageAlign(align: ImageAlignments) {
  setImageAlign(align)
}
function handleFlipX() {
  const image = props.editor.getAttributes('image')
  const { flipX } = image
  props.editor
    .chain()
    .focus(undefined, { scrollIntoView: false })
    .updateImage({
      flipX: !flipX,
    })
    .run()
}
function handleFlipY() {
  const image = props.editor.getAttributes('image')
  const { flipY } = image
  props.editor
    .chain()
    .focus(undefined, { scrollIntoView: false })
    .updateImage({
      flipY: !flipY,
    })
    .run()
}

function handleRemove() {
  const { state, dispatch } = props.editor.view
  deleteSelection(state, dispatch)
}
</script>
<template>
  <BubbleMenu
    :editor="editor"
    :should-show="shouldShow"
    :tippy-options="{
      offset: [0, 8],
      zIndex: 10,
      popperOptions: {
        modifiers: [{ name: 'flip', enabled: false }],
      },
      appendTo: 'parent',
      getReferenceClientRect: getReferenceClientRect.value,
      plugins: [sticky],
      sticky: 'popper',
    }"
    :update-delay="0"
    plugin-key="image-menus-123"
  >
    <div
      class="border border-neutral-200 dark:border-neutral-800 px-3 py-2 transition-all select-none pointer-events-auto shadow-sm rounded-sm w-auto bg-background"
    >
      <div class="flex items-center flex-nowrap whitespace-nowrap h-[26px] justify-start relative gap-0.5">
        <ActionButton
:action="handleFlipX"
:tooltip="t('composer.image.menu.flipX')"
icon="FlipVertical" />
        <ActionButton
:action="handleFlipY"
:tooltip="t('composer.image.menu.flipY')"
icon="FlipHorizontal" />
        <Separator
class="mx-1 me-2 h-[16px]"
orientation="vertical" />
        <Popover>
          <PopoverTrigger>
            <ActionButton
:title="t('composer.image.menu.size')"
icon="ImageSize" />
          </PopoverTrigger>
          <PopoverContent class="w-84">
            <div class="flex items-center gap-2">
              <Label
class="whitespace-nowrap"
for="maxWidth">{{ t('composer.image.menu.size.width') }}</Label>
              <Input
                id="maxWidth"
                v-model="width"
                class="w-20 h-8 [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                type="number"
                @input="updateHeightFromWidth"
                @keyup.enter="updateImageSize"
              />
              <Label
class="whitespace-nowrap"
for="maxWidth">{{ t('composer.image.menu.size.height') }}</Label>
              <Input
                id="maxWidth"
                v-model="height"
                class="w-20 h-8 [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                type="number"
                @input="updateWidthFromHeight"
                @keyup.enter="updateImageSize"
              />
            </div>
            <div class="mt-3">
              <Tabs
                v-model:model-value="imagePercent"
                @update:model-value="
                  value => {
                    imagePercent = value as string
                  }
                "
              >
                <TabsList>
                  <TabsTrigger
v-for="value in ['25', '50', '75', '100']"
:key="value"
:value="value">
                    {{ value }}%
                  </TabsTrigger>
                </TabsList>
              </Tabs>
            </div>
          </PopoverContent>
        </Popover>
        <Separator
class="mx-1 me-2 h-[16px]"
orientation="vertical" />
        <ActionButton
          v-for="(item, index) in imageAlign"
          :key="index"
          :action="() => handleSetImageAlign(item)"
          :disabled="!editor.can().setTextAlign(item)"
          :icon="alignIconMap[item]"
          :is-active="() => editor.isActive({ textAlign: item }) || false"
          :tooltip="t(`editor.textalign.${item}.tooltip`)"
        />
        <Separator
class="mx-1 me-2 h-[16px]"
orientation="vertical" />
        <ActionButton
          :action="handleRemove"
          :disabled="!editor.isEditable"
          :tooltip="t('composer.remove')"
          icon="Trash2"
        />
      </div>
    </div>
  </BubbleMenu>
</template>
