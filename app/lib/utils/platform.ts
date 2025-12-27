export function isMac(): boolean {
  return /apple/i.test(navigator.vendor)
}

export function getShortcutKey(key: string): string {
  switch (key.toLowerCase()) {
    case 'meta':
    case 'mod':
      return isMac() ? '⌘' : 'Ctrl'
    case 'alt':
      return isMac() ? '⌥' : 'Alt'
    case 'control':
      return isMac() ? '⌃' : 'Ctrl'
    case 'shift':
      return isMac() ? '⇧' : 'Shift'
    case 'enter':
    case 'return':
      return '↵'
    case 'backspace':
      return isMac() ? '⌫' : 'Backspace'
    case 'delete':
      return isMac() ? '⌦' : 'Delete'
    case 'arrowup':
      return '↑'
    case 'arrowdown':
      return '↓'
    case 'arrowleft':
      return '←'
    case 'arrowright':
      return '→'
    case 'pageup':
      return '⇞'
    case 'pagedown':
      return '⇟'
    case 'home':
      return '↖'
    case 'end':
      return '↘'
    case 'escape':
      return 'Esc'
    case 'space':
      return '␣'
    case 'tab':
      return '⇥'
    default:
      return key.toUpperCase()
  }
}

export function getShortcutKeys(keys: string[]): string {
  return keys.map(getShortcutKey).join(' ')
}