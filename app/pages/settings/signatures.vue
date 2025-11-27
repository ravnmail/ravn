<script lang="ts" setup>
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '~/components/ui/card'
import { Label } from '~/components/ui/label'
import { Button } from '~/components/ui/button'
import { Separator } from '~/components/ui/separator'
import { Input } from '~/components/ui/input'
import { Textarea } from '~/components/ui/textarea'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '~/components/ui/dialog'
import type { Signature, SignaturesSettings } from '~/types/settings'

const { t } = useI18n()
const { settings: signaturesSettings, isLoading, updateCategory } = useCategorySettings('signatures')

const localSettings = ref<SignaturesSettings | null>(null)
const editingSignature = ref<Signature | null>(null)
const isDialogOpen = ref(false)
const isEditing = ref(false)

// Form state for dialog
const formTitle = ref('')
const formContent = ref('')
const formAccounts = ref('')

watch(signaturesSettings, (newSettings) => {
  if (newSettings && !localSettings.value) {
    localSettings.value = JSON.parse(JSON.stringify(newSettings))
  }
}, { immediate: true })

const hasChanges = computed(() => {
  if (!localSettings.value || !signaturesSettings.value) return false
  return JSON.stringify(localSettings.value) !== JSON.stringify(signaturesSettings.value)
})

const saveChanges = async () => {
  if (!localSettings.value) return
  try {
    await updateCategory(localSettings.value)
  } catch (err) {
    console.error('Failed to save signatures:', err)
  }
}

const resetChanges = () => {
  if (signaturesSettings.value) {
    localSettings.value = JSON.parse(JSON.stringify(signaturesSettings.value))
  }
}

const openAddDialog = () => {
  isEditing.value = false
  editingSignature.value = null
  formTitle.value = ''
  formContent.value = ''
  formAccounts.value = ''
  isDialogOpen.value = true
}

const openEditDialog = (signature: Signature) => {
  isEditing.value = true
  editingSignature.value = signature
  formTitle.value = signature.title
  formContent.value = signature.content
  formAccounts.value = signature.defaultForAccounts.join(', ')
  isDialogOpen.value = true
}

const handleSaveSignature = () => {
  if (!localSettings.value) return

  const accounts = formAccounts.value
    .split(',')
    .map(a => a.trim())
    .filter(a => a.length > 0)

  if (isEditing.value && editingSignature.value) {
    // Update existing signature
    const index = localSettings.value.items.findIndex(s => s.id === editingSignature.value!.id)
    if (index !== -1) {
      localSettings.value.items[index] = {
        ...editingSignature.value,
        title: formTitle.value,
        content: formContent.value,
        defaultForAccounts: accounts,
      }
    }
  } else {
    // Add new signature
    const newSignature: Signature = {
      id: crypto.randomUUID(),
      title: formTitle.value,
      content: formContent.value,
      defaultForAccounts: accounts,
    }
    localSettings.value.items.push(newSignature)
  }

  isDialogOpen.value = false
}

const handleDeleteSignature = (id: string) => {
  if (!localSettings.value) return
  if (confirm(t('pages.signatures.actions.delete'))) {
    localSettings.value.items = localSettings.value.items.filter(s => s.id !== id)

    // Clear global default if this was it
    if (localSettings.value.globalDefault === id) {
      localSettings.value.globalDefault = null
    }
  }
}

const setGlobalDefault = (id: string) => {
  if (!localSettings.value) return
  localSettings.value.globalDefault = id
}
</script>

<template>
  <div class="space-y-6">
    <!-- Page Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-bold tracking-tight">{{ t('pages.signatures.title') }}</h2>
        <p class="text-muted-foreground">
          {{ t('pages.signatures.description') }}
        </p>
      </div>
      <Button @click="openAddDialog">
        <Icon
          class="mr-2"
          name="lucide:plus"
        />
        {{ t('pages.signatures.addButton') }}
      </Button>
    </div>

    <Separator/>

    <!-- Signatures List -->
    <Card v-if="localSettings">
      <CardHeader>
        <CardTitle>{{ t('pages.signatures.yourSignatures.title') }}</CardTitle>
        <CardDescription>
          {{ t('pages.signatures.yourSignatures.description') }}
        </CardDescription>
      </CardHeader>

      <CardContent>
        <div
          v-if="localSettings.items.length === 0"
          class="py-12 text-center"
        >
          <Icon
            class="mx-auto size-12 text-muted-foreground/50"
            name="lucide:pen-tool"
          />
          <p class="mt-4 text-sm text-muted-foreground">
            {{ t('pages.signatures.emptyState') }}
          </p>
        </div>

        <div
          v-else
          class="space-y-4"
        >
          <div
            v-for="signature in localSettings.items"
            :key="signature.id"
            class="rounded-lg border p-4 space-y-3"
          >
            <!-- Signature Header -->
            <div class="flex items-start justify-between">
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <h3 class="font-medium">{{ signature.title }}</h3>
                  <Badge
                    v-if="localSettings.globalDefault === signature.id"
                    variant="secondary"
                  >
                    {{ t('pages.signatures.globalDefault') }}
                  </Badge>
                </div>
                <p
                  v-if="signature.defaultForAccounts.length > 0"
                  class="text-sm text-muted-foreground"
                >
                  {{ t('pages.signatures.defaultFor') }} {{ signature.defaultForAccounts.join(', ') }}
                </p>
              </div>
              <div class="flex gap-2">
                <Button
                  size="icon"
                  :title="t('pages.signatures.actions.edit')"
                  variant="ghost"
                  @click="openEditDialog(signature)"
                >
                  <Icon
                    name="lucide:pencil"
                  />
                </Button>
                <Button
                  size="icon"
                  :title="t('pages.signatures.actions.delete')"
                  variant="ghost"
                  @click="handleDeleteSignature(signature.id)"
                >
                  <Icon
                    class="text-destructive"
                    name="lucide:trash-2"
                  />
                </Button>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex gap-2">
              <Button
                v-if="localSettings.globalDefault !== signature.id"
                size="sm"
                variant="outline"
                @click="setGlobalDefault(signature.id)"
              >
                {{ t('pages.signatures.actions.setGlobalDefault') }}
              </Button>
            </div>
          </div>
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
            class="mr-2 animate-spin"
            name="lucide:loader-2"
          />
          {{ t('common.actions.saveChanges') }}
        </Button>
      </CardFooter>
    </Card>

    <!-- Add/Edit Dialog -->
    <Dialog v-model:open="isDialogOpen">
      <DialogContent class="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>{{ isEditing ? t('pages.signatures.dialog.titleEdit') : t('pages.signatures.dialog.titleAdd') }}</DialogTitle>
          <DialogDescription>
            {{ isEditing ? t('pages.signatures.dialog.descriptionEdit') : t('pages.signatures.dialog.descriptionAdd') }}
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <!-- Title -->
          <div class="space-y-2">
            <Label for="title">{{ t('pages.signatures.form.titleLabel') }}</Label>
            <Input
              id="title"
              v-model="formTitle"
              :placeholder="t('pages.signatures.form.titlePlaceholder')"
            />
            <p class="text-sm text-muted-foreground">
              {{ t('pages.signatures.form.titleHelper') }}
            </p>
          </div>

          <!-- Content -->
          <div class="space-y-2">
            <Label for="content">{{ t('pages.signatures.form.contentLabel') }}</Label>
            <Textarea
              id="content"
              v-model="formContent"
              :placeholder="t('pages.signatures.form.contentPlaceholder')"
              rows="6"
            />
            <p class="text-sm text-muted-foreground">
              {{ t('pages.signatures.form.contentHelper') }}
            </p>
          </div>

          <!-- Default For Accounts -->
          <div class="space-y-2">
            <Label for="accounts">{{ t('pages.signatures.form.accountsLabel') }}</Label>
            <Input
              id="accounts"
              v-model="formAccounts"
              :placeholder="t('pages.signatures.form.accountsPlaceholder')"
            />
            <p class="text-sm text-muted-foreground">
              {{ t('pages.signatures.form.accountsHelper') }}
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            @click="isDialogOpen = false"
          >
            {{ t('common.actions.cancel') }}
          </Button>
          <Button
            :disabled="!formTitle || !formContent"
            @click="handleSaveSignature"
          >
            {{ isEditing ? t('pages.signatures.actions.update') : t('pages.signatures.actions.create') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Loading State -->
    <Card v-if="isLoading && !localSettings">
      <CardContent class="py-12 text-center">
        <Icon
          class="mx-auto size-8 animate-spin text-muted-foreground"
          name="lucide:loader-2"
        />
        <p class="mt-4 text-sm text-muted-foreground">{{ t('pages.signatures.loading') }}</p>
      </CardContent>
    </Card>
  </div>
</template>
