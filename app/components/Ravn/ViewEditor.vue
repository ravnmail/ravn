<script lang="ts" setup>
import type {
  View,
  CreateViewRequest,
  UpdateViewRequest,
  KanbanSwimlane,
  ViewConfig
} from '~/types/view'
import { Button } from '~/components/ui/button'
import IconNameField from '~/components/ui/IconNameField.vue'

const props = defineProps<{
  viewId?: string | null
}>()

const emit = defineEmits<{
  close: []
  saved: [view: View]
}>()

const { t } = useI18n()
const { views, createView, updateView, getView, addSwimlane, updateSwimlane, deleteSwimlane } = useViews()
const { labels } = useLabels()
const { getFolders } = useFolders()
const { getAccounts } = useAccounts()

const isDialogOpen = defineModel<boolean>('open', { default: false })
const isLoading = ref(false)
const currentView = ref<View | null>(null)
const allFolders = ref<any[]>([])

const formData = ref<CreateViewRequest | UpdateViewRequest>({
  name: '',
  view_type: 'kanban',
  icon: undefined,
  color: undefined,
  folders: [] as string[],
  config: {
    type: 'kanban',
    swimlanes: [] as KanbanSwimlane[]
  }
})

const newSwimlane = ref({
  title: '',
  color: '#3B82F6',
  label_ids: [] as string[],
  folder_ids: [] as string[]
})

// Group folders by account for easier selection
const foldersByAccount = computed(() => {
  const grouped = new Map<string, { account: any, folders: any[] }>()

  allFolders.value.forEach(folder => {
    const accountId = folder.account_id
    if (!grouped.has(accountId)) {
      grouped.set(accountId, {
        account: { id: accountId, name: folder.account_name || `Account ${accountId}` },
        folders: []
      })
    }
    grouped.get(accountId)!.folders.push(folder)
  })

  return Array.from(grouped.values())
})

const loadData = async () => {
  try {
    // Load folders from all accounts
    const accounts = await getAccounts()
    const folderPromises = accounts.map(account =>
      getFolders(account.id).catch(err => {
        console.error(`Failed to load folders for account ${account.id}:`, err)
        return []
      })
    )
    const accountFolders = await Promise.all(folderPromises)
    allFolders.value = accountFolders.flat()

    // Load view data if editing
    if (props.viewId) {
      const view = await getView(props.viewId)
      if (view) {
        currentView.value = view
        formData.value = {
          name: view.name,
          view_type: view.view_type,
          icon: view.icon,
          color: view.color,
          folders: view.folders,
          config: view.config,
          sort_order: view.sort_order
        }
      }
    }
  } catch (error) {
    console.error('Failed to load data:', error)
  }
}

// Watch for dialog open
watch(() => isDialogOpen.value, async (open, wasOpen) => {
  if (open && !wasOpen) {
    // Dialog is opening - load data
    await loadData()
  } else if (!open && wasOpen) {
    // Dialog is closing - reset form
    resetForm()
  }
})

onMounted(async () => {
  if (isDialogOpen.value) {
    await loadData()
  }
})

const resetForm = () => {
  formData.value = {
    name: '',
    icon: null,
    color: null,
    view_type: 'kanban',
    folders: [],
    config: {
      type: 'kanban',
      swimlanes: []
    }
  }
  currentView.value = null
}

const addSwimlaneToForm = () => {
  if (!newSwimlane.value.title.trim()) return

  const swimlane: KanbanSwimlane = {
    id: crypto.randomUUID(),
    title: newSwimlane.value.title.trim(),
    color: newSwimlane.value.color,
    label_ids: newSwimlane.value.label_ids,
    folder_ids: newSwimlane.value.folder_ids.length > 0 ? newSwimlane.value.folder_ids : undefined,
    state: 'open',
    sort_order: formData.value.config.swimlanes.length
  }

  formData.value.config.swimlanes.push(swimlane)

  newSwimlane.value = {
    title: '',
    color: '#3B82F6',
    label_ids: [],
    folder_ids: []
  }
}

const removeSwimlane = (index: number) => {
  formData.value.config.swimlanes.splice(index, 1)
  formData.value.config.swimlanes.forEach((s, i) => {
    s.sort_order = i
  })
}

const moveSwimlane = (from: number, to: number) => {
  const swimlanes = [...formData.value.config.swimlanes]
  const [moved] = swimlanes.splice(from, 1)
  swimlanes.splice(to, 0, moved)
  swimlanes.forEach((s, i) => {
    s.sort_order = i
  })
  formData.value.config.swimlanes = swimlanes
}

const handleSubmit = async () => {
  if (!formData.value.name.trim()) return
  if (formData.value.config.swimlanes.length === 0) {
    alert(t('components.viewEditor.swimlanes.emptyState'))
    return
  }

  isLoading.value = true
  try {
    if (currentView.value) {
      const view = await updateView({ ...formData.value, id: currentView.value.id } as UpdateViewRequest)
      emit('saved', view)
    } else {
      const view = await createView(formData.value as CreateViewRequest)
      emit('saved', view)
    }

    isDialogOpen.value = false
    resetForm()
  } catch (error) {
    console.error('Failed to save view:', error)
  } finally {
    isLoading.value = false
  }
}

const toggleFolder = (folderId: string) => {
  const index = formData.value.folders.indexOf(folderId)
  if (index > -1) {
    formData.value.folders.splice(index, 1)
  } else {
    formData.value.folders.push(folderId)
  }
}

const toggleLabel = (labelId: string) => {
  const index = newSwimlane.value.label_ids.indexOf(labelId)
  if (index > -1) {
    newSwimlane.value.label_ids.splice(index, 1)
  } else {
    newSwimlane.value.label_ids.push(labelId)
  }
}

const toggleFolderForAccount = (accountId: string, folderId: string) => {
  // Remove any existing folder for this account
  const accountFolders = foldersByAccount.value.find(a => a.account.id === accountId)?.folders || []
  const existingFolderIds = accountFolders.map(f => String(f.id))
  newSwimlane.value.folder_ids = newSwimlane.value.folder_ids.filter(
    id => !existingFolderIds.includes(id)
  )

  // Add the new folder if not already selected
  const folderIdStr = String(folderId)
  if (!newSwimlane.value.folder_ids.includes(folderIdStr)) {
    newSwimlane.value.folder_ids.push(folderIdStr)
  }
}

const getSelectedFolderForAccount = (accountId: string) => {
  const accountFolders = foldersByAccount.value.find(a => a.account.id === accountId)?.folders || []
  const accountFolderIds = accountFolders.map(f => String(f.id))
  return newSwimlane.value.folder_ids.find(id => accountFolderIds.includes(id))
}

const getLabelName = (labelId: string) => {
  return labels.value.find(l => l.id === labelId)?.name || 'Unknown'
}

const getFolderName = (folderId: string) => {
  return allFolders.value.find(f => String(f.id) === folderId)?.name || 'Unknown'
}
</script>

<template>
  <UiDialog v-model:open="isDialogOpen">
    <UiDialogContent class="max-w-4xl max-h-[90vh] overflow-y-auto">
      <UiDialogHeader>
        <UiDialogTitle>
          {{ currentView ? t('components.viewEditor.title.edit') : t('components.viewEditor.title.create') }}
        </UiDialogTitle>
        <UiDialogDescription>
          {{ t('components.viewEditor.description') }}
        </UiDialogDescription>
      </UiDialogHeader>

      <div class="space-y-6 py-4">
        <IconNameField
          :model-value="formData"
          @update:model-value="Object.assign(formData, $event)"
        />

        <!-- Folders Selection -->
        <div class="space-y-2">
          <label class="text-sm font-medium">{{ t('components.viewEditor.includeFolders') }}</label>
          <div class="border rounded-lg p-3 space-y-2 max-h-48 overflow-y-auto">
            <div
              v-for="folder in allFolders"
              :key="folder.id"
              class="flex items-center gap-2"
            >
              <UiCheckbox
                :checked="formData.folders.includes(String(folder.id))"
                @update:model-value="toggleFolder(String(folder.id))"
              />
              <span class="text-sm">{{ folder.name }}</span>
            </div>
            <div
              v-if="allFolders.length === 0"
              class="text-center py-4 text-gray-500 text-sm"
            >
              {{ t('components.viewEditor.noFolders') }}
            </div>
          </div>
        </div>

        <!-- Swimlanes -->
        <div class="space-y-2">
          <label class="text-sm font-medium">{{ t('components.viewEditor.swimlanes.title') }}</label>

          <!-- Add New Swimlane -->
          <div class="border rounded-lg p-4 space-y-3 bg-gray-50 dark:bg-gray-900">
            <IconNameField
              :model-value="{ ...newSwimlane, name: newSwimlane.title }"
              @update:model-value="(e) => newSwimlane = { ...e, title: e.name }"
            />

            <!-- Label Selection for Swimlane -->
            <div class="space-y-2">
              <label class="text-xs text-gray-600 dark:text-gray-400">
                {{ t('components.viewEditor.swimlanes.labels') }}
              </label>
              <div class="flex flex-wrap gap-2">
                <button
                  v-for="label in labels"
                  :key="label.id"
                  :class="newSwimlane.label_ids.includes(label.id)
                    ? 'bg-gray-900 text-white dark:bg-gray-100 dark:text-gray-900'
                    : 'bg-gray-200 dark:bg-gray-800'"
                  class="px-3 py-1 rounded-full text-xs transition-all"
                  type="button"
                  @click="toggleLabel(label.id)"
                >
                  <span
                    :style="{ backgroundColor: label.color || '#3B82F6' }"
                    class="inline-block w-2 h-2 rounded-full mr-1"
                  />
                  {{ label.name }}
                </button>
              </div>
            </div>

            <!-- Folder Selection for Swimlane -->
            <div class="space-y-2">
              <label class="text-xs text-gray-600 dark:text-gray-400">
                {{ t('components.viewEditor.swimlanes.folders') }}
              </label>
              <div class="space-y-3">
                <div
                  v-for="accountGroup in foldersByAccount"
                  :key="accountGroup.account.id"
                  class="border rounded-lg p-3 space-y-2"
                >
                  <div class="text-xs font-medium text-gray-700 dark:text-gray-300">
                    {{ accountGroup.account.name }}
                  </div>
                  <div class="flex flex-wrap gap-2">
                    <button
                      v-for="folder in accountGroup.folders"
                      :key="folder.id"
                      :class="getSelectedFolderForAccount(accountGroup.account.id) === String(folder.id)
                        ? 'bg-gray-900 text-white dark:bg-gray-100 dark:text-gray-900'
                        : 'bg-gray-200 dark:bg-gray-800'"
                      class="px-3 py-1 rounded-full text-xs transition-all"
                      type="button"
                      @click="toggleFolderForAccount(accountGroup.account.id, String(folder.id))"
                    >
                      {{ folder.name }}
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <UiButton
              :disabled="!newSwimlane.title.trim()"
              size="sm"
              @click="addSwimlaneToForm"
            >
              <Icon
                class="mr-2 h-4 w-4"
                name="lucide:plus"
              />
              {{ t('components.viewEditor.actions.addSwimlane') }}
            </UiButton>
          </div>

          <!-- Existing Swimlanes -->
          <div class="space-y-2">
            <div
              v-for="(swimlane, index) in formData.config.swimlanes"
              :key="swimlane.id"
              class="flex items-center gap-3 px-1 border border-border rounded-lg bg-gray-950"
            >
              <div class="flex flex-col">
                <Button
                  :disabled="index === 0"
                  size="icon"
                  @click="moveSwimlane(index, index - 1)"
                >
                  <Icon name="lucide:chevron-up"/>
                </Button>
                <Button
                  :disabled="index === formData.config.swimlanes.length - 1"
                  size="icon"
                  @click="moveSwimlane(index, index + 1)"
                >
                  <Icon name="lucide:chevron-down"/>
                </Button>
              </div>

              <div
                :style="{ backgroundColor: swimlane.color }"
                class="size-4 rounded-full flex-shrink-0"
              />

              <div class="flex-1">
                <div class="font-medium">{{ swimlane.title }}</div>
                <div
                  v-if="swimlane.label_ids.length > 0"
                  class="text-xs text-muted"
                >
                  {{ t('components.viewEditor.swimlanes.labels') }}: {{ swimlane.label_ids.map(getLabelName).join(', ') }}
                </div>
                <div
                  v-if="swimlane.folder_ids && swimlane.folder_ids.length > 0"
                  class="text-xs text-muted"
                >
                  {{ t('components.viewEditor.swimlanes.folders') }}: {{ swimlane.folder_ids.map(getFolderName).join(', ') }}
                </div>
              </div>

              <UiButton
                size="sm"
                variant="ghost"
                @click="removeSwimlane(index)"
              >
                <Icon
                  class="h-4 w-4 text-red-500"
                  name="lucide:trash-2"
                />
              </UiButton>
            </div>

            <div
              v-if="formData.config.swimlanes.length === 0"
              class="text-center p-8 text-gray-500 border rounded-lg"
            >
              {{ t('components.viewEditor.swimlanes.emptyState') }}
            </div>
          </div>
        </div>
      </div>

      <UiDialogFooter>
        <UiButton
          variant="outline"
          @click="isDialogOpen = false"
        >
          {{ t('common.actions.cancel') }}
        </UiButton>
        <UiButton
          :disabled="!formData.name.trim() || formData.config.swimlanes.length === 0 || isLoading"
          @click="handleSubmit"
        >
          <Icon
            v-if="isLoading"
            class="mr-2 h-4 w-4 animate-spin"
            name="lucide:loader-2"
          />
          {{ currentView ? t('components.viewEditor.actions.updateView') : t('components.viewEditor.actions.createView') }}
        </UiButton>
      </UiDialogFooter>
    </UiDialogContent>
  </UiDialog>
</template>
