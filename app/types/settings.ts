// Settings Types
// These correspond to the settings in src-tauri/resources/settings.json5

import type { CleanTranslation } from 'nuxt-i18n-micro-types'

export interface AIModelSettings {
  fast: string;
  normal: string;
}

export interface AISettings {
  models: AIModelSettings;
}

// Signature Interface
export interface Signature {
  id: string;
  title: string;
  content: string;
  defaultForAccounts: string[];
}

export interface SignaturesSettings {
  items: Signature[];
  globalDefault: string | null;
}

// Keyboard Bindings
export interface KeyboardBindings {
  [action: string]: string[];
}

export interface KeyboardSettings {
  enabled: boolean;
  bindings: KeyboardBindings;
}

export interface EmailSettings {
  renderMode: 'simple' | 'normal';
}

// Root settings interface
export interface Settings {
  ai: AISettings;
  signatures: SignaturesSettings;
  keyboard: KeyboardSettings;
  email: EmailSettings;
}

// Navigation item for settings sidebar
export interface SettingsNavItem {
  title: CleanTranslation | string;
  name: string;
  icon: string;
  badge?: string | number;
  disabled?: boolean;
}

// For partial updates
export type PartialSettings = Partial<Settings>;
export type PartialDeep<T> = {
  [P in keyof T]?: T[P] extends object ? PartialDeep<T[P]> : T[P];
};
