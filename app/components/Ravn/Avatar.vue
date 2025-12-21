<script lang="ts" setup>

import { convertFileSrc } from '@tauri-apps/api/core'

const props = withDefaults(defineProps<{
  size?: 'xs' | 'sm' | 'md' | 'lg',
  src?: string,
  name?: string,
  email: string,
}>(), {
  src: undefined,
  name: undefined,
  size: 'md'
})
const imageError = ref(false)

const { useGetContactByEmail } = useContacts()
const { data: contact } = useGetContactByEmail(props.email)

const getInitialsFromName = (name: string) => {
  const parts = name.split(' ')
  if (parts.length >= 2) {
    return `${parts[0][0]}${parts[parts.length - 1][0]}`.toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
}

const initials = computed(() => {
  if (props.name) {
    return getInitialsFromName(props.name)
  }

  if (contact.value?.first_name && contact.value?.last_name) {
    return `${contact.value.first_name[0]}${contact.value.last_name[0]}`.toUpperCase()
  }

  if (contact.value?.display_name) {
    return getInitialsFromName(contact.value.display_name)
  }

  return props.email?.slice(0, 2).toUpperCase()
})

const shouldShowAvatar = computed(() => {
  return avatarUrl.value && !imageError.value
})

const avatarUrl = computed(() => contact.value?.avatar_path ? convertFileSrc(contact.value.avatar_path) : props.src)

const handleImageError = (e: Error) => {
  imageError.value = !!e
}

const classes = computed(() => {
  return [
    shouldShowAvatar.value ? 'bg-white' : 'bg-muted/30',
    {
      xs: 'size-4 text-[0.5rem] rounded-sm',
      sm: 'size-5 text-[0.5rem] rounded',
      md: 'w-8 h-8 text-xs rounded-xl',
      lg: 'w-10 h-10 text-base rounded-xl',
    }[props.size]
  ]
})

</script>

<template>
  <div
    :class="['overflow-clip text-muted-foreground flex items-center justify-center font-bold relative shrink-0', classes]"
  >
    <img
      v-if="shouldShowAvatar"
      :src="avatarUrl!"
      class="w-full h-full object-cover"
      @error="handleImageError"
    >
    <span v-else>{{ initials }}</span>
  </div>
</template>

