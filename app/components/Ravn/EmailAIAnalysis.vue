<script lang="ts" setup>
import type { EmailDetail, EmailAnalysis } from '~/types/email'
import { Button } from '~/components/ui/button'

defineProps<{
  email: EmailDetail
  analysis?: EmailAnalysis | null
  isAnalyzing?: boolean
  error?: string | null
}>()

const emit = defineEmits<{
  quickReply: [content: string]
}>()

const { t } = useI18n()

const handleResponseClick = (response: { title: string, content: string }) => {
  emit('quickReply', response.content)
}
</script>

<template>
  <div class="bg-surface rounded-lg overflow-hidden border-l-3 border-ai">
    <div
      v-if="isAnalyzing"
      class="flex items-center space-x-2 text-sm text-gray-400 p-3 text-ai"
    >
      <Icon
        class="w-4 h-4 animate-spin"
        name="lucide:loader-2"
      />
      <span>{{ t('components.aiAnalysis.analyzing') }}</span>
    </div>
    <div
      v-if="error && !isAnalyzing"
      class="p-3"
    >
      <div class="flex items-start space-x-3 text-red-400">
        <Icon
          class="w-5 h-5 flex-shrink-0 mt-0.5"
          name="lucide:alert-circle"
        />
        <div>
          <p class="font-medium">
            {{ t('components.aiAnalysis.failed') }}
          </p>
          <p class="text-sm text-gray-400 mt-1">
            {{ error }}
          </p>
        </div>
      </div>
    </div>
    <div
      v-else-if="analysis && !isAnalyzing"
      class="p-3 space-y-4"
    >
      <div class="gap-3 text-foreground select-auto">{{ analysis.gist }}</div>
      <div v-if="analysis.responses && analysis.responses.length > 0">
        <div class="flex flex-wrap gap-2">
          <Button
            v-for="(response, index) in analysis.responses"
            :key="index"
            size="sm"
            variant="outline"
            @click="handleResponseClick(response)"
          >
            {{ response.title }}
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
