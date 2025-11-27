<script lang="ts" setup>

const route = useRoute()
const folderId = computed(() => route.params.folder_id as string)
const accountId = computed(() => route.params.account_id as string)

</script>

<template>
  <div class="flex h-screen w-full select-none">
    <RavnMailList
      :account-id="accountId"
      :folder-id="folderId"
      class="h-full overflow-y-scroll shrink-0"
    />
    <NuxtPage/>
  </div>
</template>