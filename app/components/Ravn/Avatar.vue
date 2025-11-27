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

const classes = computed(() => {
  return [
    {
      xs: 'size-4 text-[0.5rem] rounded-sm',
      sm: 'size-5 text-[0.5rem] rounded',
      md: 'w-8 h-8 text-xs rounded-xl',
      lg: 'w-10 h-10 text-base rounded-xl',
    }[props.size]
  ]
})

const getInitialsFromName = (name: string) => {
  const parts = name.split(' ')
  if (parts.length >= 2) {
    return `${parts[0][0]}${parts[parts.length - 1][0]}`.toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
}

const initials = computed(() => {
  if (contact.value?.first_name && contact.value?.last_name) {
    return `${contact.value.first_name[0]}${contact.value.last_name[0]}`.toUpperCase()
  }

  if (contact.value?.display_name) {
    return getInitialsFromName(contact.value.display_name)
  }

  if (props.name) {
    return getInitialsFromName(props.name)
  }

  return props.email.slice(0, 2).toUpperCase()
})

const shouldShowAvatar = computed(() => {
  return avatarUrl.value && !imageError.value
})

const avatarUrl = computed(() => contact.value ? convertFileSrc(contact.value.avatar_path) : props.src)

const handleImageError = (e: Error) => {
  imageError.value = !!e
}

</script>

<template>
  <div
    :class="['overflow-clip bg-muted text-muted-foreground flex items-center justify-center font-bold relative shrink-0', classes]"
  >
    <img
      v-if="shouldShowAvatar"
      :alt="name || email"
      :src="avatarUrl!"
      class="w-full h-full object-cover bg-white"
      @error="handleImageError"
    >
    <span v-else>{{ initials }}</span>
  </div>
</template>

