import { invoke } from '@tauri-apps/api/core'
import { Extension } from '@tiptap/core'
import debounce from 'lodash/debounce'
import { Plugin, PluginKey } from 'prosemirror-state'
import { Decoration, DecorationSet } from 'prosemirror-view'

export interface AutocompleteContactNote {
  email: string
  display_name?: string | null
  notes: string
}

export interface AutocompleteEmailMetadata {
  sender: string
  subject: string
  is_reply: boolean
  recipients: string[]
}

interface AutocompleteOptions {
  autoTriggerEnabled: boolean
  triggerThreshold: number
  triggerDelay: number
  completionClass: string
  /** Live email metadata (sender, recipients, subject) provided by the Composer */
  emailMetadata?: () => AutocompleteEmailMetadata
  /** AI notes for the current recipients, provided by the Composer */
  contactNotes?: () => AutocompleteContactNote[]
  /** The prior/quoted email body, provided by the Composer */
  priorEmail?: () => string | undefined
}

interface EmailContext {
  metadata: {
    sender: string
    subject: string
    is_reply: boolean
    recipients: string[]
  }
  prior_email?: string
  current_text: string
  cursor_position: number
  contact_notes?: AutocompleteContactNote[]
}

const autocompletePluginKey = new PluginKey('autocomplete')

export const Autocomplete = Extension.create<AutocompleteOptions>({
  name: 'autocomplete',

  addOptions() {
    return {
      autoTriggerEnabled: true,
      triggerThreshold: 15,
      triggerDelay: 500,
      completionClass: 'opacity-50 italic',
    }
  },

  addStorage() {
    return {
      suggestion: '',
      isWaitingForSuggestion: false,
    }
  },

  addKeyboardShortcuts() {
    return {
      Tab: () => this.editor.commands.acceptSuggestion(),
      Escape: () => this.editor.commands.clearSuggestion(),
    }
  },

  addCommands() {
    return {
      acceptSuggestion:
        () =>
        ({ editor }) => {
          const { suggestion } = this.storage
          if (!suggestion) {
            return false
          }

          const { state, view } = editor
          view.dispatch(state.tr.insertText(suggestion))
          this.storage.suggestion = ''

          view.dispatch(
            state.tr.setMeta(autocompletePluginKey, {
              type: 'setSuggestion',
              suggestion: '',
            })
          )

          return true
        },

      clearSuggestion:
        () =>
        ({ editor }) => {
          this.storage.suggestion = ''

          const { state, view } = editor
          view.dispatch(
            state.tr.setMeta(autocompletePluginKey, {
              type: 'setSuggestion',
              suggestion: '',
            })
          )

          return true
        },

      triggerSuggestion:
        () =>
        ({ editor }) => {
          const { state, view } = editor
          view.dispatch(
            state.tr.setMeta(autocompletePluginKey, {
              type: 'requestSuggestion',
            })
          )
          return true
        },
    }
  },

  addProseMirrorPlugins() {
    const extension = this
    const pluginOptions = this.options

    return [
      new Plugin({
        key: autocompletePluginKey,

        state: {
          init() {
            return DecorationSet.empty
          },

          apply(tr, oldState, oldPluginState, newState) {
            const mappedDecorations = oldState.map(tr.mapping, tr.doc)

            const meta = tr.getMeta(autocompletePluginKey)
            if (meta) {
              if (meta.type === 'setSuggestion') {
                if (meta.suggestion) {
                  const position = tr.selection.$head.pos
                  const decoration = Decoration.widget(
                    position,
                    () => {
                      const span = document.createElement('span')
                      span.className = pluginOptions.completionClass
                      span.textContent = meta.suggestion
                      return span
                    },
                    { side: 1, key: 'autocomplete' }
                  )

                  return DecorationSet.create(tr.doc, [decoration])
                } else {
                  return DecorationSet.empty
                }
              } else if (meta.type === 'requestSuggestion') {
                requestSuggestion(extension, newState)
              }
            }

            if (tr.selectionSet) {
              return DecorationSet.empty
            }

            return mappedDecorations
          },
        },

        props: {
          decorations(state) {
            return this.getState(state)
          },
        },

        view() {
          const debouncedSuggest = debounce(async (state) => {
            await requestSuggestion(extension, state)
          }, pluginOptions.triggerDelay)

          console.log('Autocomplete plugin initialized with options:', pluginOptions)

          return {
            update(view, prevState) {
              const { state } = view
              if (!pluginOptions.autoTriggerEnabled) return

              if (prevState.doc.content !== state.doc.content) {
                const textLength = getTextLength(state)
                if (textLength >= pluginOptions.triggerThreshold) {
                  debouncedSuggest(state)
                } else if (extension.storage.suggestion) {
                  extension.storage.suggestion = ''
                  view.dispatch(
                    state.tr.setMeta(autocompletePluginKey, {
                      type: 'setSuggestion',
                      suggestion: '',
                    })
                  )
                }
              }
            },

            destroy() {
              debouncedSuggest.cancel()
            },
          }
        },
      }),
    ]
  },
})

function getTextUpToCursor(state) {
  const { selection } = state
  const { from } = selection

  let text = ''
  state.doc.nodesBetween(0, from, (node) => {
    if (node.isText) {
      text += node.text
    } else if (node.isBlock && text.length > 0) {
      text += '\n'
    }
    return true
  })

  return text
}

function getTextLength(state) {
  return getTextUpToCursor(state).length
}

async function requestSuggestion(extension, state) {
  if (extension.storage.isWaitingForSuggestion) return
  extension.storage.isWaitingForSuggestion = true

  try {
    const currentText = getTextUpToCursor(state)

    const opts: AutocompleteOptions = extension.options
    const liveMetadata = opts.emailMetadata?.()
    const contactNotes = opts.contactNotes?.()
    const priorEmail = opts.priorEmail?.()

    const emailContext: EmailContext = {
      metadata: {
        sender: liveMetadata?.sender ?? '',
        subject: liveMetadata?.subject ?? '',
        is_reply: liveMetadata?.is_reply ?? false,
        recipients: liveMetadata?.recipients ?? [],
      },
      prior_email: priorEmail ?? '',
      current_text: currentText,
      cursor_position: state.selection.from,
      contact_notes: contactNotes?.length ? contactNotes : undefined,
    }

    console.log('requesting autocomplete with context:', emailContext)

    const result = await invoke('generate_email_completion', {
      context: emailContext,
    })

    if (result.error) {
      console.error('Autocomplete error:', result.error)
      return
    }

    if (result.completion) {
      console.log('completed', result)
      extension.storage.suggestion = result.completion

      const tr = state.tr.setMeta(autocompletePluginKey, {
        type: 'setSuggestion',
        suggestion: result.completion,
      })

      const view = extension.editor.view
      view.dispatch(tr)
    }
  } catch (error) {
    console.error('Autocomplete request failed:', error)
  } finally {
    extension.storage.isWaitingForSuggestion = false
  }
}

export const autocompleteStyles = `
.autocomplete-suggestion {
  opacity: 0.6;
  color: #888;
  font-style: italic;
  pointer-events: none;
}
`
