import TipTapPlaceholder from '@tiptap/extension-placeholder'

export const Placeholder = TipTapPlaceholder.configure({
  placeholder: ({ node }) => {
    if (node.type.name === 'heading') {
      return 'composer.placeholders.heading'
    }
    if (node.type.name === 'codeBlock') {
      return 'composer.placeholders.code'
    }

    return 'composer.placeholders.default'
  },
  includeChildren: true,
})
