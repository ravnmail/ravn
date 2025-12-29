<script lang="ts" setup>

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '~/components/ui/table'
import Shortcuts from '~/components/ui/kbd/Shortcuts.vue'
import { ScrollArea } from '~/components/ui/scroll-area'
import { InputField } from '~/components/ui/form'
import { Dialog, DialogContent, DialogFooter, DialogHeaderCombined } from '~/components/ui/dialog'
import type { KeybindingListItem } from '~/composables/useKeybindings'
import { Button } from '~/components/ui/button'
import IconName from '~/components/ui/IconName.vue'
import KeystrokeRecorder from '~/components/ui/form/KeystrokeRecorder.vue'
import { clone } from 'lodash'

const { keybindingsList, setKeybinding, removeKeybinding } = useKeybindings()

const searchTerm = ref('')
const selectedKeybinding = ref<KeybindingListItem | null>(null)

const showEditDialog = computed({
  get: () => selectedKeybinding.value !== null,
  set: (val: boolean) => {
    if (!val) {
      selectedKeybinding.value = null
    }
  }
})

const filteredKeybindings = computed(() => {
  if (!searchTerm.value) {
    return keybindingsList.value
  }

  const search = searchTerm.value.toLowerCase()
  return keybindingsList.value.filter((kb) => {
    return kb.fullName.toLowerCase().includes(search)
      || kb.context.toLowerCase().includes(search)
      || (kb.action && kb.action.toLowerCase().includes(search))
  })
})

const save = () => {
  if (selectedKeybinding.value) {
    setKeybinding(
      selectedKeybinding.value.context,
      selectedKeybinding.value.key,
      selectedKeybinding.value.action,
      selectedKeybinding.value.props
    )
    selectedKeybinding.value = null
  }
}

const trash = (keybinding: KeybindingListItem) => {
  if (keybinding.source === 'User') {
    removeKeybinding(keybinding.context, keybinding.key)
  } else {
    setKeybinding(keybinding.context, keybinding.key, null, null)
  }
}

</script>

<template>
  <div class="flex flex-1 h-full w-full flex-col">
    <div class="p-3">
      <IconName
        :name="$t('pages.keymapEditor.title')"
        class="text-primary pt-1"
        icon="keyboard"
      />
    </div>
    <div class="px-3">
      <InputField
        v-model="searchTerm"
        name="search"
        placeholder="Search Keybindings"
      />
    </div>
    <ScrollArea
      class="p-3"
      direction="vertical"
    >
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead/>
            <TableHead>Action</TableHead>
            <TableHead>Arguments</TableHead>
            <TableHead>Keystrokes</TableHead>
            <TableHead>Context</TableHead>
            <TableHead>Source</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="(keybinding, index) in filteredKeybindings"
            :key="index"
            @dblclick="selectedKeybinding = keybinding"
          >
            <TableCell class="space-x-px">
              <Button
                v-if="keybinding.action"
                tabindex="-1"
                variant="ghost"
                size="none"
                @click="selectedKeybinding = clone(keybinding)"
              >
                <Icon name="lucide:pencil"/>
              </Button>
              <Button
                class="hover:text-destructive"
                tabindex="-1"
                variant="ghost"
                size="none"
                @click="trash(keybinding)"
              >
                <Icon name="lucide:trash-2"/>
              </Button>
            </TableCell>
            <TableCell :class="[keybinding.action === null ? 'opacity-50' : '']">{{ keybinding.fullName }}</TableCell>
            <TableCell :class="[keybinding.action === null ? 'opacity-50' : '']">
              {{ keybinding.props ? JSON.stringify(keybinding.props) : 'N/A' }}
            </TableCell>
            <TableCell :class="[keybinding.action === null ? 'opacity-50' : '']">
              <Shortcuts :keys="keybinding.key"/>
            </TableCell>
            <TableCell :class="[keybinding.action === null || keybinding.context === 'global' ? 'opacity-50' : '']">{{ keybinding.context.replace('global', '<global>') }}</TableCell>
            <TableCell :class="[keybinding.action === null ? 'opacity-50' : '']">{{ keybinding.source }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </ScrollArea>
    <Dialog v-model:open="showEditDialog">
      <DialogContent class="max-w-lg overflow-y-auto">
        <DialogHeaderCombined
          :description="selectedKeybinding?.description"
          :title="selectedKeybinding?.fullName"
        />
        <div
          v-if="selectedKeybinding"
          class="flex flex-col gap-4"
        >
          <KeystrokeRecorder
            v-model="selectedKeybinding.key"
            label="Keystrokes"
            name="keystrokes"
          />
          <InputField
            v-model="selectedKeybinding.context"
            label="Context"
            name="context"
          />
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            @click="showEditDialog = false"
          >{{ $t('common.actions.cancel') }}
          </Button>
          <Button @click="save">{{ $t('common.actions.save') }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>