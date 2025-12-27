<script lang="ts" setup>

import { useElementHover, useMouse, useTimeoutFn, useFocusWithin } from '@vueuse/core'
import { Button } from '~/components/ui/button'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '~/components/ui/resizable'
import { SimpleTooltip } from '~/components/ui/tooltip'

const stickySidebar = ref<boolean>(true)
const showSidebar = ref<boolean>(true)

const { register, unregister, executeAction, getAction } = useActions()
// refs for hover targets
const btnRef = useTemplateRef<HTMLElement | null>('btnRef')
const isHoveringBtn = useElementHover(btnRef)
const leftOffset = ref(16)
const sidebarRef = useTemplateRef<HTMLElement | null>('sidebarRef')
const { focused } = useFocusWithin(sidebarRef)

function hideSidebar() {
  if (!stickySidebar.value) {
    showSidebar.value = false
    leftOffset.value = 16
  }
}

const { x } = useMouse()
const isNearLeft = computed(() => (x.value ?? Infinity) < leftOffset.value) // widen threshold to 48px

const hideDelay = 500
const hideTimer = useTimeoutFn(hideSidebar, hideDelay)

watch(focused, (isFocused) => {
  if (isFocused) {
    hideTimer.stop()
    leftOffset.value = 260
    showSidebar.value = true
  } else if (!stickySidebar.value) {
    hideTimer.start()
  }
}, { immediate: true })

watch([isHoveringBtn, isNearLeft], ([hBtn, near]) => {
  if (hBtn || near) {
    hideTimer.stop()
    leftOffset.value = 260
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

register({
  id: 'toggleSidebarSticky',
  namespace: 'global',
  handler: () => {
    collapseSidebar(stickySidebar.value)
  }
})

const toggleAction = getAction('global', 'toggleSidebarSticky')
const searchAction = getAction('global', 'search')
const composeAction = getAction('global', 'composeEmail')

onBeforeUnmount(() => {
  unregister('global', 'toggleSidebarSticky')
})

</script>

<template>
  <ResizablePanelGroup
    auto-save-id="default-layout-sidebar"
    class="flex min-h-screen w-screen"
    direction="horizontal"
  >
    <div class="fixed top-1 left-21 z-20">
      <SimpleTooltip
        :shortcut="toggleAction?.shortcut"
        :tooltip-markdown="toggleAction?.tooltip"
      >
        <Button
          ref="btnRef"
          size="bar"
          variant="ghost"
          @click="collapseSidebar(stickySidebar)"
        >
          <Icon name="lucide:sidebar"/>
        </Button>
      </SimpleTooltip>
      <template
        v-if="!stickySidebar"
      >
        <SimpleTooltip
          :shortcut="searchAction?.shortcut"
          :tooltip-markdown="searchAction?.tooltip"
        >
          <Button
            size="bar"
            variant="ghost"
            @click="executeAction('global', 'search')"
          >
            <Icon name="lucide:search"/>
          </Button>
        </SimpleTooltip>
        <SimpleTooltip
          :shortcut="composeAction?.shortcut"
          :tooltip-markdown="composeAction?.tooltip"
        >
          <Button
            size="bar"
            variant="ghost"
            @click="executeAction('global', 'composeEmail')"
          >
            <Icon
              class="text-primary"
              name="lucide:square-pen"
            />
          </Button>
        </SimpleTooltip>
      </template>
    </div>
    <ResizablePanel
      id="sidebar-panel"
      ref="sidebarRef"
      :collapsed-size="0"
      :default-size="250"
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
    <ResizableHandle
      @dblclick="collapseSidebar(stickySidebar)"
    />
    <ResizablePanel
      id="main-panel"
    >
      <slot/>
    </ResizablePanel>
  </ResizablePanelGroup>
</template>