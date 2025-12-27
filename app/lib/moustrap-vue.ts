type EventType = 'keypress' | 'keydown' | 'keyup'
type KeyCallback = (event?: KeyboardEvent) => void
type SequenceCallback = (sequence: string[]) => void

const KEY_MAP: Record<number, string> = {
  8: 'backspace', 9: 'tab', 13: 'enter', 16: 'shift', 17: 'ctrl',
  18: 'alt', 20: 'capslock', 27: 'esc', 32: 'space', 33: 'pageup',
  34: 'pagedown', 35: 'end', 36: 'home', 37: 'left', 38: 'up',
  39: 'right', 40: 'down', 45: 'ins', 46: 'del', 91: 'meta', 93: 'meta', 224: 'meta',
}

const KEYCODE_MAP: Record<number, string> = {
  106: '*', 107: '+', 109: '-', 110: '.', 111: '/',
  186: ';', 187: '=', 188: ',', 189: '-', 190: '.',
  191: '/', 192: '`', 219: '[', 220: '\\', 221: ']', 222: "'",
}

for (let i = 1; i < 20; i++) KEY_MAP[111 + i] = `f${i}`
for (let i = 0; i <= 9; i++) KEY_MAP[i + 96] = i.toString()

interface KeyBinding {
  callback: KeyCallback
  action: EventType
}

interface BindingEntry {
  isSequence: boolean
  binding: KeyBinding
}

interface RecordingState {
  currentKeys: string[]
  recordedSequence: string[][]
  characterKey: boolean
  timer: ReturnType<typeof setTimeout> | null
  callback: SequenceCallback | null
}

interface MousetrapState {
  recording: boolean
  currentRecordedKeys: string[]
  recordedSequence: string[][]
  sequenceKeys: string[]
  sequenceTimeout: number
}

class MousetrapEngine {
  private bindings: Map<string, BindingEntry> = new Map()
  private reverseMap: Record<string, string> | null = null
  private recordingState: RecordingState
  private sequenceTimer: ReturnType<typeof setTimeout> | null = null
  private readonly state: MousetrapState

  constructor(private target: Element | Document) {
    this.state = reactive({
      recording: false,
      currentRecordedKeys: [],
      recordedSequence: [],
      sequenceKeys: [],
      sequenceTimeout: 1000,
    })

    this.recordingState = {
      currentKeys: [],
      recordedSequence: [],
      characterKey: false,
      timer: null,
      callback: null,
    }

    this.attachEventListeners()
  }

  static addKeycodes(mappings: Record<number, string>): void {
    Object.assign(KEY_MAP, mappings)
  }

  bind(keys: string | string[], callback: KeyCallback, action?: EventType): this {
    const keyArray = Array.isArray(keys) ? keys : [keys]

    keyArray.forEach((keyCombo) => {
      if (keyCombo.includes(' ')) {
        this.bindSequence(keyCombo, callback)
      } else {
        this.bindSingleKey(keyCombo, callback, action)
      }
    })

    return this
  }

  unbind(keys: string | string[], action?: EventType): this {
    const keyArray = Array.isArray(keys) ? keys : [keys]

    keyArray.forEach((keyCombo) => {
      if (keyCombo.includes(' ')) {
        this.bindings.delete(keyCombo)
        this.bindings.delete(`${keyCombo}:keydown`)
      } else {
        this.unbindSingleKey(keyCombo, action)
      }
    })

    return this
  }

  trigger(keys: string, action?: EventType): this {
    if (keys.includes(' ')) {
      this.triggerSequence(keys)
    } else {
      this.triggerSingleKey(keys, action)
    }
    return this
  }

  record(callback: SequenceCallback): this {
    this.state.recording = true
    this.recordingState.callback = callback
    this.recordingState.recordedSequence = []
    this.recordingState.currentKeys = []
    this.recordingState.characterKey = false
    this.state.currentRecordedKeys = []
    this.state.recordedSequence = []
    return this
  }

  stopRecording(): void {
    if (this.recordingState.timer) {
      clearTimeout(this.recordingState.timer)
    }
    this.finishRecording()
  }

  reset(): this {
    this.bindings.clear()
    this.state.recording = false
    this.clearSequence()
    this.state.currentRecordedKeys = []
    this.state.recordedSequence = []
    this.recordingState = {
      currentKeys: [],
      recordedSequence: [],
      characterKey: false,
      timer: null,
      callback: null,
    }
    return this
  }

  setSequenceTimeout(ms: number): this {
    this.state.sequenceTimeout = ms
    return this
  }

  getState(): MousetrapState {
    return this.state
  }

  private attachEventListeners(): void {
    this.target.addEventListener('keydown', (e) => this.handleKey(e as KeyboardEvent, 'keydown'))
    this.target.addEventListener('keyup', (e) => this.handleKey(e as KeyboardEvent, 'keyup'))
    this.target.addEventListener('keypress', (e) => this.handleKey(e as KeyboardEvent, 'keypress'))
  }

  private bindSequence(keyCombo: string, callback: KeyCallback): void {
    const binding: KeyBinding = { callback, action: 'keydown' }
    this.bindings.set(keyCombo, { isSequence: true, binding })
    this.bindings.set(`${keyCombo}:keydown`, { isSequence: true, binding })
  }

  private bindSingleKey(keyCombo: string, callback: KeyCallback, action?: EventType): void {
    const keyParts = this.parseKeyCombo(keyCombo)
    const character = keyParts[keyParts.length - 1] || ''
    const modifiers = keyParts.slice(0, -1)
    const finalAction = this.determineAction(character, modifiers, action)

    const normalizedCombo = modifiers.length > 0
      ? this.normalizeModifiers([...modifiers, character])
      : keyCombo

    const binding: KeyBinding = { callback, action: finalAction }
    this.bindings.set(`${normalizedCombo}:${finalAction}`, { isSequence: false, binding })

    if (modifiers.length === 0) {
      this.bindings.set(normalizedCombo, { isSequence: false, binding })
    }
  }

  private mapKeyCombo(keyCombo: string, action: 'keypress' | 'keydown' | 'keyup' | undefined) {
    const keyParts = this.parseKeyCombo(keyCombo)
    const character = keyParts[keyParts.length - 1] || ''
    const modifiers = keyParts.slice(0, -1)
    const finalAction = action || this.determineAction(character, modifiers)

    const normalizedCombo = modifiers.length > 0
      ? this.normalizeModifiers([...modifiers, character])
      : keyCombo
    return { modifiers, finalAction, normalizedCombo }
  }

  private unbindSingleKey(keyCombo: string, action?: EventType): void {
    const { modifiers, finalAction, normalizedCombo } = this.mapKeyCombo(keyCombo, action)

    this.bindings.delete(`${normalizedCombo}:${finalAction}`)
    if (modifiers.length === 0) {
      this.bindings.delete(normalizedCombo)
    }
  }

  private triggerSequence(keys: string): void {
    const entry = this.bindings.get(keys) || this.bindings.get(`${keys}:keydown`)
    entry?.binding.callback()
  }

  private triggerSingleKey(keys: string, action?: EventType): void {
    const { modifiers, finalAction, normalizedCombo } = this.mapKeyCombo(keys, action)

    let entry = this.bindings.get(`${normalizedCombo}:${finalAction}`)
    if (!entry && modifiers.length === 0) {
      entry = this.bindings.get(normalizedCombo)
    }

    entry?.binding.callback()
  }

  private handleKey(e: KeyboardEvent, eventType: EventType): void {
    if (this.state.recording) {
      this.recordKey(e, eventType)
      return
    }

    if (this.shouldIgnoreEvent(e)) {
      return
    }

    const character = this.extractCharacter(e)
    const modifiers = this.extractModifiers(e)

    if (this.handleSequenceKey(character, modifiers, eventType, e)) {
      return
    }

    const normalizedCombo = this.normalizeModifiers([...modifiers, character])
    this.state.sequenceKeys.push(normalizedCombo)

    if (this.tryMatchBinding(normalizedCombo, eventType, e)) {
      return
    }

    if (this.couldContinueSequence()) {
      this.resetSequenceTimer()
    } else {
      this.clearSequence()
    }
  }

  private handleSequenceKey(
    character: string,
    modifiers: string[],
    eventType: EventType,
    e: KeyboardEvent
  ): boolean {
    if (this.state.sequenceKeys.length === 0) return false

    const lastKey = this.state.sequenceKeys[this.state.sequenceKeys.length - 1]
    if (lastKey !== character) return false

    this.state.sequenceKeys.push(character)

    if (this.tryMatchBinding(character, eventType, e)) {
      return true
    }

    if (this.couldContinueSequence()) {
      this.resetSequenceTimer()
    } else {
      this.clearSequence()
    }

    return true
  }

  private tryMatchBinding(
    keyCombo: string,
    eventType: EventType,
    e: KeyboardEvent
  ): boolean {
    if (this.state.sequenceKeys.length === 1) {
      return this.tryMatchSingleBinding(keyCombo, eventType, e)
    }

    return this.tryMatchSequenceBinding(e)
  }

  private tryMatchSingleBinding(
    keyCombo: string,
    eventType: EventType,
    e: KeyboardEvent
  ): boolean {
    const mapKey = `${keyCombo}:${eventType}`
    let entry = this.bindings.get(mapKey)

    if (entry && !entry.isSequence) {
      this.clearSequence()
      entry.binding.callback(e)
      return true
    }

    entry = this.bindings.get(keyCombo)
    if (entry && !entry.isSequence) {
      this.clearSequence()
      entry.binding.callback(e)
      return true
    }

    return false
  }

  private tryMatchSequenceBinding(e: KeyboardEvent): boolean {
    const sequenceStr = this.state.sequenceKeys.join(' ')
    let sequenceEntry = this.bindings.get(sequenceStr)

    if (!sequenceEntry) {
      sequenceEntry = this.bindings.get(`${sequenceStr}:keydown`)
    }

    if (sequenceEntry?.isSequence) {
      this.clearSequence()
      sequenceEntry.binding.callback(e)
      return true
    }

    return false
  }

  private couldContinueSequence(): boolean {
    const currentSeq = this.state.sequenceKeys.join(' ')

    for (const [key] of this.bindings) {
      const bindKey = key.split(':')[0] || ''

      if (bindKey.includes(' ') && bindKey.startsWith(currentSeq) && bindKey !== currentSeq) {
        return true
      }
    }

    return false
  }

  private recordKey(e: KeyboardEvent, eventType: EventType): void {
    const character = this.extractCharacter(e)
    const modifiers = this.extractModifiers(e)

    if (eventType === 'keydown') {
      if (character.length === 1 && this.recordingState.characterKey) {
        this.commitRecordedCombo()
      }

      modifiers.forEach((mod) => this.addRecordedKey(mod))
      this.addRecordedKey(character)
    } else if (eventType === 'keyup' && this.recordingState.currentKeys.length > 0) {
      this.commitRecordedCombo()
    }
  }

  private addRecordedKey(key: string): void {
    if (this.recordingState.currentKeys.includes(key)) {
      return
    }

    this.recordingState.currentKeys.push(key)

    if (key.length === 1) {
      this.recordingState.characterKey = true
    }

    this.state.currentRecordedKeys = [...this.recordingState.currentKeys]
  }

  private commitRecordedCombo(): void {
    if (this.recordingState.currentKeys.length === 0) return

    this.recordingState.recordedSequence.push([...this.recordingState.currentKeys])
    this.recordingState.currentKeys = []
    this.recordingState.characterKey = false
    this.state.currentRecordedKeys = []
    this.state.recordedSequence = this.recordingState.recordedSequence.map((combo) => [...combo])

    this.scheduleRecordingTimeout()
  }

  private scheduleRecordingTimeout(): void {
    if (this.recordingState.timer) {
      clearTimeout(this.recordingState.timer)
    }

    this.recordingState.timer = setTimeout(() => this.finishRecording(), 1000)
  }

  private finishRecording(): void {
    if (this.recordingState.currentKeys.length > 0) {
      this.commitRecordedCombo()
    }

    if (this.recordingState.callback) {
      const normalizedSequence = this.recordingState.recordedSequence.map((combo) => {
        return combo
          .sort((a, b) => {
            if (a.length > 1 && b.length === 1) return -1
            if (a.length === 1 && b.length > 1) return 1
            return a > b ? 1 : -1
          })
          .join('+')
      })
      this.recordingState.callback(normalizedSequence)
    }

    this.state.recording = false
    this.recordingState.recordedSequence = []
    this.recordingState.currentKeys = []
    this.recordingState.characterKey = false
    this.recordingState.callback = null
    this.state.currentRecordedKeys = []
    this.state.recordedSequence = []
  }

  private clearSequence(): void {
    if (this.sequenceTimer) {
      clearTimeout(this.sequenceTimer)
    }
    this.state.sequenceKeys = []
    this.sequenceTimer = null
  }

  private resetSequenceTimer(): void {
    if (this.sequenceTimer) {
      clearTimeout(this.sequenceTimer)
    }

    this.sequenceTimer = setTimeout(() => {
      this.clearSequence()
    }, this.state.sequenceTimeout)
  }

  private shouldIgnoreEvent(e: KeyboardEvent): boolean {
    const element = e.target as Element

    if (element && element.classList?.contains('mousetrap')) {
      return false
    }

    if (['INPUT', 'SELECT', 'TEXTAREA'].includes(element?.tagName)) {
      return true
    }

    return element && (element as HTMLElement).isContentEditable
  }

  private extractCharacter(e: KeyboardEvent): string {
    const w = e.which
    if (e.type === 'keypress') {
      let character = String.fromCharCode(w)
      if (!e.shiftKey) {
        character = character.toLowerCase()
      }
      return character
    }

    if (KEY_MAP[w]) {
      return KEY_MAP[w]!
    }

    if (KEYCODE_MAP[w]) {
      return KEYCODE_MAP[w]!
    }

    return String.fromCharCode(w).toLowerCase()
  }

  private extractModifiers(e: KeyboardEvent): string[] {
    const modifiers: string[] = []

    if (e.shiftKey) modifiers.push('shift')
    if (e.altKey) modifiers.push('alt')
    if (e.ctrlKey) modifiers.push('ctrl')
    if (e.metaKey) modifiers.push('meta')

    return modifiers
  }

  private getReverseMap(): Record<string, string> {
    if (!this.reverseMap) {
      this.reverseMap = {}
      for (const key in KEY_MAP) {
        const keyNum = parseInt(key)
        if (keyNum > 95 && keyNum < 112) continue

        if (Object.prototype.hasOwnProperty.call(KEY_MAP, keyNum)) {
          this.reverseMap[KEY_MAP[keyNum]!] = key
        }
      }
    }
    return this.reverseMap
  }

  private determineAction(
    key: string,
    modifiers: string[],
    action?: EventType
  ): EventType {
    if (!action) {
      action = this.getReverseMap()[key] ? 'keydown' : 'keypress'
    }

    if (action === 'keypress' && modifiers.length) {
      action = 'keydown'
    }

    return action
  }

  private parseKeyCombo(combination: string): string[] {
    if (combination === '+') return ['+']

    combination = combination.replace(/\+{2}/g, '+plus')
    return combination.split('+')
  }

  private normalizeModifiers(keys: string[]): string {
    return keys
      .sort((a, b) => {
        if (a.length > 1 && b.length === 1) return -1
        if (a.length === 1 && b.length > 1) return 1
        return a > b ? 1 : -1
      })
      .join('+')
  }
}

let engineInstance: MousetrapEngine | null = null

export function useMousetrap(element?: Element | Document) {
  const target = element || document

  if (!engineInstance) {
    engineInstance = new MousetrapEngine(target)
  }

  const state = engineInstance.getState()

  return {
    bind: engineInstance.bind.bind(engineInstance),
    unbind: engineInstance.unbind.bind(engineInstance),
    trigger: engineInstance.trigger.bind(engineInstance),
    reset: engineInstance.reset.bind(engineInstance),
    record: engineInstance.record.bind(engineInstance),
    stopRecording: engineInstance.stopRecording.bind(engineInstance),
    setSequenceTimeout: engineInstance.setSequenceTimeout.bind(engineInstance),

    currentRecordedKeys: computed(() => state.currentRecordedKeys),
    recordedSequence: computed(() => state.recordedSequence),
    isRecording: computed(() => state.recording),
  }
}

export { MousetrapEngine }
export type { KeyCallback, SequenceCallback, EventType }