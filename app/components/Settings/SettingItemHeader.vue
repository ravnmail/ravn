<script lang="ts" setup>
import { SimpleTooltip } from '~/components/ui/tooltip'
import { Button } from '~/components/ui/button'

import { useClipboard } from '@vueuse/core'
import type { SettingGroup } from '~/types/settings-manifest'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'

const { isUserSetting } = useSettings()

const props = defineProps<{
  title: CleanTranslation
  description: CleanTranslation
  settingKey: string
  group: SettingGroup
}>()

const emit = defineEmits<{
  reset: []
}>()

const { t } = useI18n()
const source = `ravn://settings/${props.group.id}#${props.settingKey}`
const { copy, copied } = useClipboard({ source })

</script>

<template>
  <div class="group flex items-top gap-2">
    <SimpleTooltip :tooltip="t('settings.actions.copyLink')">
      <Button
        class="opacity-0 transition-opacity group-hover:opacity-100"
        size="2xs"
        tabindex="-1"
        variant="ghost"
        @click="copy()"
      >
        <Icon
          v-if="copied"
          class="text-success"
          name="lucide:check"
        />
        <Icon
          v-else
          name="lucide:link-2"
        />
      </Button>
    </SimpleTooltip>
    <div>
      <div>
        <label class="text-primary">{{ title }}</label>
        <SimpleTooltip
          v-if="isUserSetting(settingKey)"
          :tooltip="t('settings.actions.reset')"
        >
          <Button
            class="ml-1"
            size="2xs"
            tabindex="-1"
            variant="ghost"
            @click="emit('reset')"
          >
            <Icon name="lucide:undo-2"/>
          </Button>
        </SimpleTooltip>
      </div>
      <p class="text-sm text-muted">
        {{ description }}
      </p>
    </div>
  </div>
</template>
