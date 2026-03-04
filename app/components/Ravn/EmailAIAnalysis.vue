<script lang="ts" setup>
import { Button } from '~/components/ui/button'
import type { EmailAnalysis, EmailDetail } from '~/types/email'

defineProps<{
  email: EmailDetail
  analysis?: EmailAnalysis | null
  isAnalyzing?: boolean
  error?: string | null
  reduced?: boolean
}>()

const emit = defineEmits<{
  quickReply: [content: string]
  regenerate: []
}>()

const { t } = useI18n()

const handleResponseClick = (response: { title: string; content: string }) => {
  emit('quickReply', response.content)
}
</script>

<template>
  <div class="relative overflow-hidden rounded-lg border-l-3 border-ai bg-surface pr-5">
    <Button
      v-if="!isAnalyzing"
      class="absolute top-2.5 right-3 h-5 w-5 text-muted hover:text-foreground"
      size="icon"
      variant="ghost"
      :title="t('components.aiAnalysis.regenerate')"
      @click="emit('regenerate')"
    >
      <Icon name="lucide:refresh-cw" />
    </Button>
    <div
      v-if="isAnalyzing"
      class="flex items-center space-x-2 px-3 py-2.5 text-sm"
    >
      <span class="ai-animate-text font-medium">{{ t('components.aiAnalysis.analyzing') }}</span>
    </div>

    <!-- Error state -->
    <div
      v-else-if="error"
      class="px-3 py-2.5"
    >
      <div class="flex items-start space-x-3 text-red-400">
        <Icon
          class="mt-0.5 h-5 w-5 shrink-0"
          name="lucide:alert-circle"
        />
        <div>
          <p class="font-medium">
            {{ t('components.aiAnalysis.failed') }}
          </p>
          <p class="mt-1 text-sm text-gray-400">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- Analysis result -->
    <div
      v-else-if="analysis"
      class="space-y-3 px-3 pt-1.5 pb-3"
    >
      <div class="gap-3 text-foreground select-auto">{{ analysis.gist }}</div>
      <div v-if="!reduced && analysis.responses && analysis.responses.length > 0">
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
