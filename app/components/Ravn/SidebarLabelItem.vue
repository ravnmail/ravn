<script lang="ts" setup>
import LabelEditDialog from '~/components/Ravn/LabelEditDialog.vue'
import { Button } from '~/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import type { DragData } from '~/composables/useDragAndDrop'
import { useDropTarget } from '~/composables/useDragAndDrop'
import type { SidebarLabelItem } from '~/composables/useSidebarNavigation'

const props = defineProps<SidebarLabelItem>()

const { t } = useI18n()
const { addLabelToEmail } = useEmails()
const { alert } = useAlertDialog()
const { useDeleteLabelMutation } = useLabels()

const { mutateAsync: deleteLabel } = useDeleteLabelMutation()

// ─── Drop target ──────────────────────────────────────────────────────────────

const itemRef = ref<HTMLElement | null>(null)

const { isOver, canDrop } = useDropTarget(itemRef, {
  getData: () => ({
    type: 'label',
    id: props.id,
    accepts: ['email', 'conversation'],
  }),
  canDrop: (data: DragData) => data.type === 'email' || data.type === 'conversation',
  onDrop: async (data: DragData) => {
    try {
      const emailIds =
        data.type === 'conversation' && data.messageIds && data.messageIds.length > 0
          ? data.messageIds
          : data.isMultiDrag && data.messageIds && data.messageIds.length > 0
            ? data.messageIds
            : data.isMultiDrag && data.selectedIds && data.selectedIds.length > 0
              ? data.selectedIds
              : [data.id]

      await Promise.all(
        Array.from(new Set(emailIds)).map((emailId) =>
          addLabelToEmail({ email_id: emailId, label_id: props.id })
        )
      )
    } catch (err) {
      console.error('[SidebarLabelItem] Failed to assign label on drop:', err)
    }
  },
})

// ─── Edit dialog ──────────────────────────────────────────────────────────────

const isEditDialogOpen = ref(false)
const isMenuOpen = ref(false)

// ─── Delete — confirm directly from the dropdown, no dialog needed ────────────

const handleDelete = async () => {
  const confirmed = await alert.confirm(t('dialogs.confirmDelete.message', { name: props.name }), {
    title: t('dialogs.confirmDelete.title'),
    confirmLabel: t('actions.delete'),
    variant: 'destructive',
  })
  if (!confirmed) return

  try {
    await deleteLabel(props.id)
    const route = useRoute()
    if (String(route.params.label_id) === props.id) {
      navigateTo('/')
    }
  } catch (err) {
    console.error('[SidebarLabelItem] Failed to delete label:', err)
  }
}
</script>

<template>
  <div
    ref="itemRef"
    :class="[
      'group relative flex items-center rounded transition-colors',
      isOver && canDrop
        ? 'bg-primary/10 ring-2 ring-primary ring-offset-1'
        : isOver && !canDrop
          ? 'ring-2 ring-destructive ring-offset-1'
          : '',
    ]"
  >
    <!-- Main nav button -->
    <button
      class="flex min-w-0 flex-1 items-center gap-1.5 overflow-hidden py-1.5 pr-1 pl-2 text-left"
      @click="navigateTo(href)"
    >
      <Icon
        :name="`lucide:${icon || 'tag'}`"
        :size="16"
        :style="{ color: color }"
        class="shrink-0"
      />
      <span class="grow truncate text-sm font-medium">{{ name }}</span>

      <!-- Drop-over indicator badge -->
      <Icon
        v-if="isOver && canDrop"
        class="shrink-0 text-primary"
        name="lucide:tag"
        :size="12"
      />
    </button>

    <!-- ⋯ context menu (visible on hover / when open) -->
    <DropdownMenu @update:open="(v) => (isMenuOpen = v)">
      <DropdownMenuTrigger as-child>
        <Button
          :class="[
            'absolute right-1 h-6 w-6 shrink-0 p-0 opacity-0 transition-opacity group-hover:opacity-60 hover:opacity-100',
            isMenuOpen ? 'opacity-100' : '',
          ]"
          size="xs"
          variant="ghost"
          @click.stop.prevent
        >
          <Icon
            name="lucide:ellipsis"
            :size="14"
          />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        <DropdownMenuItemRich
          :label="t('components.sidebarLabelItem.actions.edit')"
          icon="lucide:pen"
          @select="isEditDialogOpen = true"
        />
        <DropdownMenuItemRich
          :label="t('components.sidebarLabelItem.actions.delete')"
          icon="lucide:trash-2"
          class="text-destructive"
          @select="handleDelete"
        />
      </DropdownMenuContent>
    </DropdownMenu>
  </div>

  <!-- Edit dialog — label data comes from the reactive props (useLabels cache) -->
  <LabelEditDialog
    v-model:open="isEditDialogOpen"
    :label="{ id, name, icon, color, created_at: '', updated_at: '' }"
    @deleted="
      () => {
        const route = useRoute()
        if (String(route.params.label_id) === id) navigateTo('/')
      }
    "
  />
</template>
