import hotkeys from 'hotkeys-js'
import { onBeforeUnmount } from 'vue'

export const useHotkeys = (keys: string, callback: CallableFunction) => {
  hotkeys.filter = () => true

  const bind = () => {
    hotkeys(keys, (e: Event) => {
      e.preventDefault()
      callback()
      return false
    })
  }

  const unbind = () => {
    hotkeys.unbind(keys)
  }

  onBeforeUnmount(() => {
    unbind()
  })

  return { bind, unbind }
}
