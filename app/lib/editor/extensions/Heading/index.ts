import type { Extension } from '@tiptap/core'
import type { HeadingOptions as TiptapHeadingOptions } from '@tiptap/extension-heading'
import { Heading as TiptapHeading } from '@tiptap/extension-heading'

import type { GeneralOptions } from '@/types/composer'

export interface HeadingOptions extends TiptapHeadingOptions, GeneralOptions<HeadingOptions> { }

export const Heading = TiptapHeading.extend<HeadingOptions>({
  addOptions() {
    return {
      ...this.parent?.(),
      levels: [1, 2, 3],
    }
  },
})
