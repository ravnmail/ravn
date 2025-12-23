<script lang="ts" setup>
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '~/components/ui/select'
import { Badge } from '~/components/ui/badge'

const { t } = useI18n()
const { useGetModels } = useCorvus()
const { data: models } = useGetModels()

const modelValue = defineModel<string | null>({
  type: [String, null],
  default: null,
})

</script>

<template>
  <Select v-model="modelValue">
    <SelectTrigger>
      <SelectValue
        :placeholder="t(String('common.select'))"
        class="flex items-center gap-2"
      >
        {{ modelValue }}
      </SelectValue>
    </SelectTrigger>
    <SelectContent>
      <SelectItem
        v-for="model in models"
        :key="model.id"
        :value="model.id"
      >
        <div class="flex items-center gap-2">
          <span>{{ model.name }}</span>
          <Badge size="sm">{{ model.id }}</Badge>
        </div>
      </SelectItem>
    </SelectContent>
  </Select>
</template>