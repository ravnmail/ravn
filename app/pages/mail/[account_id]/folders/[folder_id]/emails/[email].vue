<script lang="ts" setup>

import EmailViewer from '~/components/Ravn/EmailViewer.vue'

const emailId = useRoute().params.email as string

</script>

<template>
  <EmailViewer
    :email-id="emailId"
    class="flex-1"
  />
</template>