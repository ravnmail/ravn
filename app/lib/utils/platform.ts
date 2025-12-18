export function isMac(): boolean {
  return true
}

export function getShortcutKey(key: string): string {
  if (key.toLowerCase() === 'mod') {
    return isMac() ? '⌘' : 'Ctrl'
  } else if (key.toLowerCase() === 'alt') {
    return isMac() ? '⌥' : 'Alt'
  } else if (key.toLowerCase() === 'shift') {
    return isMac() ? '⇧' : 'Shift'
  } else if (key.toLowerCase() === 'enter') {
    return '⏎'
  } else if (key.toLowerCase() === 'backspace') {
    return '⌫'
  } else if (key.toLowerCase() === 'escape') {
    return '⎋'
  } else if (key.toLowerCase() === 'space') {
    return '␣'
  } else {
    return key
  }
}

export function getShortcutKeys(keys: string[]): string {
  return keys.map(getShortcutKey).join(' ')
}