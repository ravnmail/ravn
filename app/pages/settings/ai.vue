<script lang="ts" setup>
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '~/components/ui/card'
import { Label } from '~/components/ui/label'
import { Button } from '~/components/ui/button'
import { Separator } from '~/components/ui/separator'
import { Input } from '~/components/ui/input'
import type { AISettings } from '~/types/settings'

const { t } = useI18n()
const { settings: aiSettings, isLoading, updateCategory } = useCategorySettings('ai')

const localSettings = ref<AISettings | null>(null)

watch(aiSettings, (newSettings) => {
  if (newSettings && !localSettings.value) {
    localSettings.value = JSON.parse(JSON.stringify(newSettings))
  }
}, { immediate: true })

const hasChanges = computed(() => {
  if (!localSettings.value || !aiSettings.value) return false
  return JSON.stringify(localSettings.value) !== JSON.stringify(aiSettings.value)
})

const saveChanges = async () => {
  if (!localSettings.value) return
  try {
    await updateCategory(localSettings.value)
  } catch (err) {
    console.error('Failed to save AI settings:', err)
  }
}

const resetChanges = () => {
  if (aiSettings.value) {
    localSettings.value = JSON.parse(JSON.stringify(aiSettings.value))
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- Page Header -->
    <div>
      <h2 class="text-2xl font-bold tracking-tight">{{ t('pages.ai.title') }}</h2>
      <p class="text-muted-foreground">
        {{ t('pages.ai.description') }}
      </p>
    </div>

    <Separator/>

    <!-- Model Configuration -->
    <Card v-if="localSettings">
      <CardHeader>
        <CardTitle>{{ t('pages.ai.models.title') }}</CardTitle>
        <CardDescription>
          {{ t('pages.ai.models.description') }}
        </CardDescription>
      </CardHeader>

      <CardContent class="space-y-6">
        <!-- Fast Model -->
        <div class="space-y-2">
          <Label>{{ t('pages.ai.models.fast.label') }}</Label>
          <Input
            v-model="localSettings.models.fast"
            :placeholder="t('pages.ai.models.fast.placeholder')"
            type="text"
          />
          <p class="text-sm text-muted-foreground">{{ t('pages.ai.models.fast.description') }}</p>
        </div>

        <!-- Normal Model -->
        <div class="space-y-2">
          <Label>{{ t('pages.ai.models.normal.label') }}</Label>
          <Input
            v-model="localSettings.models.normal"
            :placeholder="t('pages.ai.models.normal.placeholder')"
            type="text"
          />
          <p class="text-sm text-muted-foreground">{{ t('pages.ai.models.normal.description') }}</p>
        </div>
      </CardContent>

      <CardFooter class="flex justify-between border-t pt-6">
        <Button
          :disabled="!hasChanges"
          variant="outline"
          @click="resetChanges"
        >
          {{ t('common.actions.resetChanges') }}
        </Button>
        <Button
          :disabled="!hasChanges || isLoading"
          @click="saveChanges"
        >
          <Icon
            v-if="isLoading"
            class="mr-2 size-4 animate-spin"
            name="lucide:loader-2"
          />
          {{ t('common.actions.saveChanges') }}
        </Button>
      </CardFooter>
    </Card>

    <!-- Loading State -->
    <Card v-if="isLoading && !localSettings">
      <CardContent class="py-12 text-center">
        <Icon
          class="mx-auto size-8 animate-spin text-muted-foreground"
          name="lucide:loader-2"
        />
        <p class="mt-4 text-sm text-muted-foreground">{{ t('pages.ai.loading') }}</p>
      </CardContent>
    </Card>
  </div>
</template>
