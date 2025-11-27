<script lang="ts" setup>

import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useElementHover, useMouse, useTimeoutFn } from '@vueuse/core'
import { Button } from '~/components/ui/button'

const stickySidebar = ref<boolean>(true)
const showSidebar = ref<boolean>(true)
const router = useRouter()

// refs for hover targets
const btnRef = useTemplateRef<HTMLElement | null>('btnRef')
const isHoveringBtn = useElementHover(btnRef)
const leftOffset = ref(16)

// use mouse x position to detect proximity to the left edge (no blocking element)
const { x } = useMouse()
const isNearLeft = computed(() => (x.value ?? Infinity) < leftOffset.value) // widen threshold to 48px

const hideDelay = 250
const hideTimer = useTimeoutFn(() => {
  showSidebar.value = false
  leftOffset.value = 16
}, hideDelay)

watch([isHoveringBtn, isNearLeft], ([hBtn, near]) => {
  if (hBtn || near) {
    hideTimer.stop()
    leftOffset.value = 256
    showSidebar.value = true
  } else if (!stickySidebar.value) {
    hideTimer.start()
  }
}, { immediate: true })

onBeforeUnmount(() => {
  hideTimer.stop()
})

</script>

<template>
  <div class="flex min-h-screen w-screen select-none">
    <div class="fixed top-1 left-21 z-20">
      <Button
        ref="btnRef"
        size="bar"
        variant="ghost"
        @click="stickySidebar = !stickySidebar"
      >
        <Icon name="lucide:sidebar"/>
      </Button>
      <template
        v-if="!stickySidebar"
      >
        <Button
          size="bar"
          variant="ghost"
          @click="router.push('/search')"
        >
          <Icon name="lucide:search"/>
        </Button>
        <Button
          size="bar"
          variant="ghost"
        >
          <Icon
            class="text-primary"
            name="lucide:square-pen"
          />
        </Button>
      </template>
    </div>
    <Sidebar
      ref="sidebarRef"
      :show="showSidebar"
      :sticky="stickySidebar"
    />
    <slot/>
  </div>
</template>