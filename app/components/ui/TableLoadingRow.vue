<script setup lang="ts">
import { TableCell, TableRow } from '~/components/ui/table'

const { $t } = useI18n()

const props = withDefaults(defineProps<{
  colspan?: number
  icon?: string
  label?: string
}>(), {
  colspan: 3,
  icon: 'lucide:loader',
  label: undefined
})

const labelText = computed(() => {
  return props.label || $t('labels.loading')
})

</script>

<template>
  <TableRow>
    <TableCell
      :colspan="colspan"
      class="py-6 text-center bg-surface"
    >
      <div class="flex flex-row justify-center items-center gap-2">
        <Icon
          :name="icon"
          class="animate-spin"
        />
        <span>{{ labelText }}</span>
      </div>
    </TableCell>
  </TableRow>
</template>

