import type { SettingsManifest } from '~/types/settings-manifest'

export const settingsManifest: SettingsManifest = [
  {
    id: 'appearance',
    name: 'settings.groups.appearance.name',
    sections: [
      {
        id: 'theme',
        name: 'settings.appearance.theme.section',
        items: [
          {
            id: 'appearance.theme',
            name: 'settings.appearance.theme.name',
            description: 'settings.appearance.theme.description',
            is: 'ThemeSelector',
          },
          {
            id: 'appearance.uiScale',
            name: 'settings.appearance.uiScale.name',
            description: 'settings.appearance.uiScale.description',
            is: 'Number',
            props: {
              min: 50,
              max: 200,
              step: 10,
            }
          }
        ],
      },
    ],
  },
  {
    id: 'ai',
    name: 'settings.groups.ai.name',
    sections: [
      {
        id: 'general',
        name: 'settings.ai.general.section',
        items: [
          {
            id: 'ai.enabled',
            name: 'settings.ai.enabled.name',
            description: 'settings.ai.enabled.description',
            is: 'Toggle',
          },
        ],
      },
      {
        id: 'api',
        name: 'settings.ai.api.section',
        items: [
          {
            id: 'ai.api.baseUrl',
            name: 'settings.ai.api.baseUrl.name',
            description: 'settings.ai.api.baseUrl.description',
            is: 'Input',
            props: {
              type: 'url',
              placeholder: 'https://api.openai.com/v1',
            },
          },
          {
            id: 'ai.api.key',
            name: 'settings.ai.api.key.name',
            description: 'settings.ai.api.key.description',
            is: 'Input',
            props: {
              type: 'password',
              placeholder: 'sk-...',
            },
          },
        ],
      },
      {
        id: 'models',
        name: 'settings.ai.models.section',
        items: [
          {
            id: 'ai.models.fast',
            name: 'settings.ai.models.fast.name',
            description: 'settings.ai.models.fast.description',
            is: 'AiModelSelector',
          },
          {
            id: 'ai.models.normal',
            name: 'settings.ai.models.normal.name',
            description: 'settings.ai.models.normal.description',
            is: 'AiModelSelector',
          },
          {
            id: 'ai.models.sorting',
            name: 'settings.ai.models.sorting.name',
            description: 'settings.ai.models.sorting.description',
            is: 'Select',
            props: {
              options: [
                { label: 'Price', value: 'price' },
                { label: 'Throughput', value: 'throughput' },
                { label: 'Latency', value: 'latency' },
              ],
            }
          }
        ],
      },
      {
        id: 'writingStyle',
        name: 'settings.ai.writingStyle.section',
        items: [
          {
            id: 'ai.writingStyle',
            name: 'settings.ai.writingStyle.name',
            description: 'settings.ai.writingStyle.description',
            is: 'Textarea',
            props: {
              autosize: true,
              rows: 8,
              cols: 64,
            }
          },
        ],
      },
      {
        id: 'prompts',
        name: 'settings.ai.prompts.section',
        items: [
          {
            id: 'ai.prompts.askAi',
            name: 'settings.ai.prompts.askAi.name',
            description: 'settings.ai.prompts.askAi.description',
            is: 'Textarea',
            props: {
              autosize: true,
              rows: 8,
              cols: 64,
            }
          },
          {
            id: 'ai.prompts.generateCompletion',
            name: 'settings.ai.prompts.generateCompletion.name',
            description: 'settings.ai.prompts.generateCompletion.description',
            is: 'Textarea',
            props: {
              autosize: true,
              rows: 8,
              cols: 64,
            }
          },
          {
            id: 'ai.prompts.generateSubject',
            name: 'settings.ai.prompts.generateSubject.name',
            description: 'settings.ai.prompts.generateSubject.description',
            is: 'Textarea',
            props: {
              autosize: true,
              rows: 8,
              cols: 64,
            }
          },
          {
            id: 'ai.prompts.analyzeEmail',
            name: 'settings.ai.prompts.analyzeEmail.name',
            description: 'settings.ai.prompts.analyzeEmail.description',
            is: 'Textarea',
            props: {
              autosize: true,
              rows: 8,
              cols: 64,
            }
          },
          {
            id: 'ai.prompts.generateSearchQuery',
            name: 'settings.ai.prompts.generateSearchQuery.name',
            description: 'settings.ai.prompts.generateSearchQuery.description',
            is: 'Textarea',
            props: {
              autosize: true,
              rows: 8,
              cols: 64,
            }
          },
        ],
      },
    ],
  },
  {
    id: 'contacts',
    name: 'settings.groups.contacts.name',
    sections: [
      {
        id: 'avatar',
        name: 'settings.contacts.avatar.section',
        items: [
          {
            id: 'contacts.avatar.services',
            name: 'settings.contacts.avatar.services.name',
            description: 'settings.contacts.avatar.services.description',
            is: 'Combobox',
            props: {
              multiple: true,
              name: 'services',
              options: [
                { label: 'Unavatar', value: 'unavatar' },
                { label: 'Favicon', value: 'favicon' },
              ],
            },
          },
        ],
      },
    ],
  },
  {
    id: 'email',
    name: 'settings.groups.email.name',
    sections: [
      {
        id: 'display',
        name: 'settings.email.display.section',
        items: [
          {
            id: 'email.renderMode',
            name: 'settings.email.renderMode.name',
            description: 'settings.email.renderMode.description',
            is: 'Select',
            props: {
              options: [
                { label: 'Simple (Markdown)', value: 'simple' },
                { label: 'Normal (HTML)', value: 'normal' },
              ],
            },
          },
        ],
      },
    ],
  },
  {
    id: 'notifications',
    name: 'settings.groups.notifications.name',
    sections: [
      {
        id: 'general',
        name: 'settings.notifications.general.section',
        items: [
          {
            id: 'notifications.enabled',
            name: 'settings.notifications.enabled.name',
            description: 'settings.notifications.enabled.description',
            is: 'Toggle',
          },
          {
            id: 'notifications.notificationFolders',
            name: 'settings.notifications.notificationFolders.name',
            description: 'settings.notifications.notificationFolders.description',
            is: 'FolderSelector',
            props: {
              multiple: true,
            },
          }
        ],
      },
      {
        id: 'badge',
        name: 'settings.notifications.badge.section',
        items: [
          {
            id: 'notifications.badgeType',
            name: 'settings.notifications.badgeType.name',
            description: 'settings.notifications.badgeType.description',
            is: 'Select',
            props: {
              options: [
                { label: 'None', value: null },
                { label: 'Unread Count', value: 'count' },
                { label: 'Dot', value: 'dot' },
              ],
            },
          },
          {
            id: 'notifications.badgeFolders',
            name: 'settings.notifications.badgeFolders.name',
            description: 'settings.notifications.badgeFolders.description',
            is: 'FolderSelector',
            props: {
              multiple: true,
            },
          }
        ],
      },
      {
        id: 'sounds',
        name: 'settings.notifications.sounds.section',
        items: [
          {
            id: 'notifications.incomingSound',
            name: 'settings.notifications.incomingSound.name',
            description: 'settings.notifications.incomingSound.description',
            is: 'Select',
            props: {
              options: [
                { label: 'Disabled', value: null },
                { label: 'Incoming 1', value: 'incoming_01' },
                { label: 'Incoming 2', value: 'incoming_02' },
              ],
            },
          },
          {
            id: 'notifications.outgoingSound',
            name: 'settings.notifications.outgoingSound.name',
            description: 'settings.notifications.outgoingSound.description',
            is: 'Select',
            props: {
              options: [
                { label: 'Disabled', value: null },
                { label: 'Outgoing 1', value: 'outgoing_01' },
                { label: 'Outgoing 2', value: 'outgoing_02' },
              ],
            },
          },
        ],
      },
    ],
  },
]
