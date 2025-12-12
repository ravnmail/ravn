<script lang="ts" setup>

import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useElementHover, useMouse, useTimeoutFn } from '@vueuse/core'
import { Button } from '~/components/ui/button'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '~/components/ui/resizable'

const stickySidebar = ref<boolean>(true)
const showSidebar = ref<boolean>(true)
const router = useRouter()

// refs for hover targets
const btnRef = useTemplateRef<HTMLElement | null>('btnRef')
const isHoveringBtn = useElementHover(btnRef)
const leftOffset = ref(16)
const sidebarRef = useTemplateRef<HTMLElement | null>('sidebarRef')

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

const onSidebarCollapse = (collapsed: boolean) => {
  stickySidebar.value = !collapsed
}

const collapseSidebar = (collapse: boolean) => {
  if (collapse) {
    sidebarRef.value.collapse()
  } else {
    sidebarRef.value.expand()
  }
}

</script>

<template>
  <ResizablePanelGroup
    auto-save-id="default-layout-sidebar"
    class="flex min-h-screen w-screen select-none"
    direction="horizontal"
  >
    <div class="fixed top-1 left-21 z-20">
      <Button
        ref="btnRef"
        size="bar"
        variant="ghost"
        @click="collapseSidebar(stickySidebar)"
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
    <ResizablePanel
      id="sidebar-panel"
      ref="sidebarRef"
      :collapsed-size="0"
      :default-size="150"
      :max-size="480"
      :min-size="240"
      collapsible
      size-unit="px"
      @collapse="() => onSidebarCollapse(true)"
      @expand="() => onSidebarCollapse(false)"
    >
      <Sidebar
        :show="showSidebar"
        :sticky="stickySidebar"
      />
    </ResizablePanel>
    <ResizableHandle/>
    <ResizablePanel
      id="main-panel"
    >
      <slot/>
    </ResizablePanel>
  </ResizablePanelGroup>
</template>