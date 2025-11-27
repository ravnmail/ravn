<script lang="ts" setup>

const isExpanded = ref(true)
const router = useRouter()

defineProps<{
  title?: string
  items: Array<{
    id: string
    name: string
    icon?: string
    color?: string
    href?: string
    click?: () => void
  }>
}>()

const handleItemClick = (item: { href?: string; click?: () => void }) => {
  if (item.click) {
    item.click()
  } else if (item.href) {
    router.push(item.href)
  }
}

</script>

<template>
  <div class="flex flex-col">
    <button
      v-if="title"
      class="text-xs font-bold uppercase text-sidebar-item-text px-2 py-1 rounded flex items-center gap-1 hover:bg-sidebar-item-hover-background hover:text-sidebar-item-hover-foreground"
      @click="isExpanded = !isExpanded"
    >
      <span>{{ title }}</span>
      <Icon
        :name="`lucide:chevron-${isExpanded ? 'down' : 'up'}`"
      />
    </button>
    <div :class="[isExpanded ? 'block' : 'hidden']">
      <div class="flex flex-col">
        <template
          v-for="item in items"
          :key="item.id"
        >
          <NuxtLink
            v-if="item.href"
            :to="item.href"
            active-class="bg-sidebar-item-active-background text-sidebar-item-active-foreground"
            class="w-full text-left overflow-hidden pl-2 pr-1 py-1.5 flex items-center gap-2 rounded hover:bg-sidebar-item-hover-background"
          >
            <Icon
              :name="`lucide:${item.icon || 'eye'}`"
              :style="{ color: item.color }"
            />
            <span class="grow text-sm font-medium">{{ item.name }}</span>
          </NuxtLink>
          <button
            v-else
            class="w-full text-left overflow-hidden pl-2 pr-1 py-1.5 flex items-center gap-2 rounded hover:bg-sidebar-item-hover-background"
            @click="() => handleItemClick(item)"
          >
            <Icon
              :name="`lucide:${item.icon || 'eye'}`"
              :style="{ color: item.color }"
              class="shrink-0 "
            />
            <span class="grow text-sm font-medium">{{ item.name }}</span>
          </button>
        </template>
      </div>
    </div>
    <slot/>
  </div>
</template>