import Input from '~/components/ui/input/Input.vue'
import { Switch } from '~/components/ui/switch'
import SelectField from '~/components/ui/form/SelectField.vue'
import ThemeSelector from '~/components/Settings/components/ThemeSelector.vue'
import AiModelSelector from '~/components/Settings/components/AiModelSelector.vue'
import UnknownSetting from '~/components/Settings/components/UnknownSetting.vue'
import ComboboxField from '~/components/ui/form/ComboboxField.vue'
import FullscreenTextField from '~/components/ui/form/FullscreenTextField.vue'
import NumberField from '~/components/ui/form/NumberField.vue'
import FolderSelection from '~/components/Ravn/FolderSelection.vue'

const componentRegistry: Record<string, Component> = {
  'AiModelSelector': AiModelSelector,
  'Combobox': ComboboxField,
  'Number': NumberField,
  'Input': Input,
  'Toggle': Switch,
  'Select': SelectField,
  'Textarea': FullscreenTextField,
  'FolderSelector': FolderSelection,
  'ThemeSelector': ThemeSelector,
  'Unknown': UnknownSetting
}

export function resolveSettingComponent(identifier: string): Component {
  const component = componentRegistry[identifier]

  if (!component) {
    return componentRegistry['Unknown']
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
  return Object.keys(componentRegistry).filter(key => key !== 'Unknown')
}
