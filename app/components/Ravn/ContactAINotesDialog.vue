<script lang="ts" setup>
import { Button } from '~/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Textarea } from '~/components/ui/textarea'
import type { Contact } from '~/types/contact'

const props = defineProps<{
  open: boolean
  email: string
  name?: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  saved: [contact: Contact]
}>()

const { useGetContactByEmail, useUpdateContactMutation, createContact } = useContacts()

const { data: contact, isLoading } = useGetContactByEmail(props.email)
const { mutateAsync: updateContact, isPending: isSaving } = useUpdateContactMutation()

const notes = ref('')
const isDirty = ref(false)

// Sync notes when contact data arrives
watch(
  contact,
  (c) => {
    if (c) {
      notes.value = c.ai_notes ?? ''
    }
  },
  { immediate: true }
)

watch(notes, () => {
  isDirty.value = true
})

const handleOpenChange = (value: boolean) => {
  emit('update:open', value)
}

const handleSave = async () => {
  try {
    let targetContact = contact.value

    if (!targetContact) {
      // Create a minimal contact entry if none exists yet
      const newId = await createContact({
        id: '',
        account_id: '',
        display_name: props.name ?? null,
        first_name: null,
        last_name: null,
        company: null,
        email: props.email,
        ai_notes: notes.value.trim() || null,
        source: 'manual',
        avatar_type: 'unprocessed',
        avatar_path: '',
        send_count: 0,
        receive_count: 0,
        last_used_at: null,
        first_seen_at: new Date().toISOString(),
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      })
      // Re-fetch is handled by cache invalidation; just close
      emit('update:open', false)
      return
    }

    const updated: Contact = {
      ...targetContact,
      ai_notes: notes.value.trim() || null,
    }

    await updateContact(updated)
    isDirty.value = false
    emit('saved', updated)
    emit('update:open', false)
  } catch (err) {
    console.error('Failed to save AI notes:', err)
  }
}

const handleClear = async () => {
  notes.value = ''
  isDirty.value = true
}

const displayName = computed(() => props.name || props.email)
const hasNotes = computed(() => notes.value.trim().length > 0)
</script>

<template>
  <Dialog
    :open="open"
    @update:open="handleOpenChange"
  >
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <div class="flex items-center gap-3">
          <RavnAvatar
            :email="email"
            :name="name ?? undefined"
          />
          <div class="min-w-0 flex-1">
            <DialogTitle class="truncate text-base leading-tight">
              {{ displayName }} Notes
            </DialogTitle>
            <DialogDescription class="mt-0.5 truncate text-sm">
              <span v-if="name">&lt;{{ email }}&gt;</span>
            </DialogDescription>
          </div>
        </div>
      </DialogHeader>

      <div class="space-y-3 py-2">
        <p class="text-sm">
          These notes are passed to the AI when composing or analysing emails involving this contact
          — for example, their communication preferences, relationship context, or topics to avoid.
        </p>
        <div
          v-if="isLoading"
          class="flex items-center justify-center py-6 text-sm text-muted"
        >
          <Icon
            class="mr-2 animate-spin"
            name="lucide:loader-2"
          />
          Loading…
        </div>

        <Textarea
          v-model="notes"
          v-else
          :placeholder="`e.g. Prefers concise replies. Works in ${displayName.split(' ')[0]}'s timezone (UTC+1). Don't discuss project X.`"
          class="min-h-32 resize-none text-sm"
        />
        <p class="text-right text-xs text-muted">{{ notes.trim().length }} characters</p>
      </div>

      <DialogFooter class="gap-2">
        <Button
          v-if="hasNotes"
          class="mr-auto"
          size="sm"
          variant="ghost"
          :disabled="isSaving"
          @click="handleClear"
        >
          <Icon name="lucide:trash-2" />
          Clear
        </Button>
        <Button
          variant="outline"
          size="sm"
          @click="handleOpenChange(false)"
        >
          Cancel
        </Button>
        <Button
          :disabled="isSaving || isLoading"
          size="sm"
          @click="handleSave"
        >
          <Icon
            v-if="isSaving"
            class="animate-spin"
            name="lucide:loader-2"
          />
          <Icon
            v-else
            name="lucide:save"
          />
          Save Notes
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
