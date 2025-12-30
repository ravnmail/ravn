<script lang="ts" setup>
import { Badge } from '~/components/ui/badge'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuTrigger
} from './ui/dropdown-menu'
import { DropdownMenuItem } from '~/components/ui/dropdown-menu'
import { Progress } from '~/components/ui/progress'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'

const { formatNumber } = useFormatting()
const { executeAction } = useActions()
const { licenseStatus, daysRemaining, aiLimits } = useLicense()

</script>

<template>
  <DropdownMenu v-if="licenseStatus?.status === 'trial'">
    <DropdownMenuTrigger as-child>
      <Badge
        :class="[$attrs.class, 'cursor-pointer']"
        variant="info"
      >
        {{ licenseStatus.status === 'trial' ? 'Trial' : 'Unlicensed' }}
      </Badge>
    </DropdownMenuTrigger>
    <DropdownMenuContent>
      <DropdownMenuLabel>License Information</DropdownMenuLabel>
      <DropdownMenuItem
        class="flex-col items-start gap-1"
        @select="() => executeAction('global:openLicenseDialog')"
      >
        <span>Free Trial</span>
        <Progress :model-value="(30 - daysRemaining) / 30 * 100"/>
        <span class="opacity-50 text-xs">{{ $t('settings.license.daysRemaining', { days: daysRemaining}) }}</span>
      </DropdownMenuItem>
      <DropdownMenuItem
        v-if="aiLimits"
        class="flex-col items-start gap-1"
        @select="() => executeAction('global:openLicenseDialog')"
      >
        <span>{{ $t('settings.license.aiOperations.title') }}</span>
        <Progress :model-value="aiLimits?.usage"/>
        <span class="opacity-50 text-xs">{{ $t('settings.license.aiOperations.percentUsed', { used: formatNumber(aiLimits.usage) }) }}</span>
      </DropdownMenuItem>
      <DropdownMenuItemRich
        icon="lucide:credit-card"
        label="Upgrade Now"
        @select="() => executeAction('global:openLicenseDialog')"
      />
    </DropdownMenuContent>
  </DropdownMenu>
</template>
