import FolderSelection from '~/components/Ravn/FolderSelection.vue'
import AiModelSelector from '~/components/Settings/components/AiModelSelector.vue'
import ReminderPresetsField from '~/components/Settings/components/ReminderPresetsField.vue'
import ThemeSelector from '~/components/Settings/components/ThemeSelector.vue'
import UnknownSetting from '~/components/Settings/components/UnknownSetting.vue'
import ComboboxField from '~/components/ui/form/ComboboxField.vue'
import FullscreenTextField from '~/components/ui/form/FullscreenTextField.vue'
import NumberField from '~/components/ui/form/NumberField.vue'
import SelectField from '~/components/ui/form/SelectField.vue'
import Input from '~/components/ui/input/Input.vue'
import { Switch } from '~/components/ui/switch'

const componentRegistry: Record<string, Component> = {
  AiModelSelector: AiModelSelector,
  Combobox: ComboboxField,
  Number: NumberField,
  Input: Input,
  Toggle: Switch,
  Select: SelectField,
  Textarea: FullscreenTextField,
  FolderSelector: FolderSelection,
  ThemeSelector: ThemeSelector,
  ReminderPresets: ReminderPresetsField,
  Unknown: UnknownSetting,
}

export function resolveSettingComponent(identifier: string): Component {
  const component = componentRegistry[identifier]
  const fallback = componentRegistry.Unknown as Component

  if (!component) {
    return fallback
  }

  return component
}

export function registerSettingComponent(identifier: string, component: Component): void {
  if (componentRegistry[identifier]) {
    console.warn(`Setting component '${identifier}' is already registered. Overwriting.`)
  }
  componentRegistry[identifier] = component
}

export function getRegisteredComponents(): string[] {
  return Object.keys(componentRegistry).filter((key) => key !== 'Unknown')
}
