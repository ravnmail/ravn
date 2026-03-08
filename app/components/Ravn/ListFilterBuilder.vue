<script lang="ts" setup>
import FolderSelection from '~/components/Ravn/FolderSelection.vue'
import LabelSelection from '~/components/Ravn/LabelSelection.vue'
import { Button } from '~/components/ui/button'
import { Checkbox } from '~/components/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select'
import type { ListFilterGroup, ListFilterOperator, Label } from '~/types/view'

type FolderOption = {
  id: string
  name: string
  icon?: string
  color?: string
}

type ListFilterRuleModel = {
  id: string
  source: 'folders' | 'labels'
  values: string[]
  operator?: 'and' | 'or'
  negated?: boolean
}

type ListFilterGroupModel = ListFilterGroup & {
  id: string
  operator: ListFilterOperator
  rules: ListFilterRuleModel[]
}

const props = withDefaults(
  defineProps<{
    modelValue: ListFilterGroupModel[]
    folders?: FolderOption[]
    labels?: Label[]
    title?: string
    description?: string
  }>(),
  {
    folders: () => [],
    labels: () => [],
    title: 'Filter groups',
    description: 'Add 1..n groups and combine folder/label rules within each group.',
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: ListFilterGroupModel[]): void
}>()

const cloneRule = (rule: Partial<ListFilterRuleModel>): ListFilterRuleModel => ({
  id: rule.id || crypto.randomUUID(),
  source: rule.source || 'folders',
  values: [...(rule.values || [])],
  operator: rule.operator || 'or',
  negated: rule.negated ?? false,
})

const createRule = (source: 'folders' | 'labels'): ListFilterRuleModel => ({
  id: crypto.randomUUID(),
  source,
  values: [],
  operator: 'or',
  negated: false,
})

const cloneGroup = (group: Partial<ListFilterGroupModel>, index = 0): ListFilterGroupModel => ({
  id: group.id || `group-${index}`,
  operator: group.operator || 'and',
  negated: group.negated ?? false,
  rules: (group.rules || []).map(cloneRule),
})

const createGroup = (): ListFilterGroupModel => ({
  id: crypto.randomUUID(),
  operator: 'and',
  rules: [],
})

const groups = computed<ListFilterGroupModel[]>(() => {
  const normalized = (props.modelValue || []).map((group, index) => cloneGroup(group, index))
  return normalized.length > 0 ? normalized : [createGroup()]
})

const emitGroups = (nextGroups: ListFilterGroupModel[]) => {
  emit(
    'update:modelValue',
    nextGroups.map((group, index) => cloneGroup(group, index))
  )
}

const updateGroup = (groupId: string, updates: Partial<ListFilterGroupModel>) => {
  emitGroups(
    groups.value.map((group) =>
      group.id === groupId ? cloneGroup({ ...group, ...updates }) : cloneGroup(group)
    )
  )
}

const addGroup = () => {
  emitGroups([...groups.value.map((group) => cloneGroup(group)), createGroup()])
}

const removeGroup = (groupId: string) => {
  const remainingGroups = groups.value.filter((group) => group.id !== groupId).map(cloneGroup)
  emitGroups(remainingGroups.length > 0 ? remainingGroups : [createGroup()])
}

const addRule = (groupId: string, source: 'folders' | 'labels') => {
  emitGroups(
    groups.value.map((group) =>
      group.id === groupId
        ? cloneGroup({
            ...group,
            rules: [...group.rules.map(cloneRule), createRule(source)],
          })
        : cloneGroup(group)
    )
  )
}

const removeRule = (groupId: string, ruleId: string) => {
  emitGroups(
    groups.value.map((group) =>
      group.id === groupId
        ? cloneGroup({
            ...group,
            rules: group.rules.filter((rule) => rule.id !== ruleId).map(cloneRule),
          })
        : cloneGroup(group)
    )
  )
}

const updateRule = (groupId: string, ruleId: string, updates: Partial<ListFilterRuleModel>) => {
  emitGroups(
    groups.value.map((group) =>
      group.id === groupId
        ? cloneGroup({
            ...group,
            rules: group.rules.map((rule) =>
              rule.id === ruleId ? cloneRule({ ...rule, ...updates }) : cloneRule(rule)
            ),
          })
        : cloneGroup(group)
    )
  )
}
</script>

<template>
  <div class="space-y-4">
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <div>
          <div class="text-sm font-medium">{{ title }}</div>
          <p class="text-muted-foreground text-xs">
            {{ description }}
          </p>
        </div>

        <Button
          size="sm"
          variant="outline"
          @click="addGroup"
        >
          <Icon
            class="mr-2 h-4 w-4"
            name="lucide:plus"
          />
          Add group
        </Button>
      </div>

      <div class="space-y-3">
        <div
          v-for="group in groups"
          :key="group.id"
          class="space-y-3 rounded-lg border bg-background p-3"
        >
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-2">
              <div class="text-sm font-medium">Group</div>

              <Select
                :model-value="group.operator"
                @update:model-value="
                  (value) =>
                    updateGroup(group.id, {
                      operator: (value as ListFilterOperator) || 'and',
                    })
                "
              >
                <SelectTrigger class="w-28">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="and">AND</SelectItem>
                  <SelectItem value="or">OR</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <Button
              size="sm"
              variant="ghost"
              @click="removeGroup(group.id)"
            >
              <Icon
                class="h-4 w-4 text-destructive"
                name="lucide:trash-2"
              />
            </Button>
          </div>

          <div class="space-y-3">
            <div
              v-for="rule in group.rules"
              :key="rule.id"
              class="space-y-2 rounded-md border bg-muted/30 p-3"
            >
              <div class="flex items-center justify-between gap-2">
                <div class="flex items-center gap-2">
                  <div class="text-sm font-medium capitalize">
                    {{ rule.source }}
                  </div>

                  <Select
                    :model-value="rule.operator || 'or'"
                    @update:model-value="
                      (value) =>
                        updateRule(group.id, rule.id, {
                          operator: (value as 'and' | 'or') || 'or',
                        })
                    "
                  >
                    <SelectTrigger class="w-24">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="and">AND</SelectItem>
                      <SelectItem value="or">OR</SelectItem>
                    </SelectContent>
                  </Select>

                  <label class="text-muted-foreground flex items-center gap-2 text-xs">
                    <Checkbox
                      :model-value="!!rule.negated"
                      @update:model-value="
                        (value) => updateRule(group.id, rule.id, { negated: !!value })
                      "
                    />
                    NOT
                  </label>
                </div>

                <Button
                  size="sm"
                  variant="ghost"
                  @click="removeRule(group.id, rule.id)"
                >
                  <Icon
                    class="h-4 w-4 text-destructive"
                    name="lucide:x"
                  />
                </Button>
              </div>

              <div v-if="rule.source === 'folders'">
                <FolderSelection
                  :model-value="rule.values"
                  @update:model-value="(v) => updateRule(group.id, rule.id, { values: v })"
                />
              </div>

              <div v-else>
                <LabelSelection
                  :model-value="rule.values"
                  @update:model-value="(v) => updateRule(group.id, rule.id, { values: v })"
                />
              </div>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button
                size="sm"
                variant="outline"
                @click="addRule(group.id, 'folders')"
              >
                <Icon
                  class="mr-2 h-4 w-4"
                  name="lucide:folder-plus"
                />
                Add folder rule
              </Button>

              <Button
                size="sm"
                variant="outline"
                @click="addRule(group.id, 'labels')"
              >
                <Icon
                  class="mr-2 h-4 w-4"
                  name="lucide:tag"
                />
                Add label rule
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
