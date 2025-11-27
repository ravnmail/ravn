import type { Editor } from "@tiptap/core";
import { Node, mergeAttributes } from "@tiptap/core";
import ActionButton from '~/components/ActionButton.vue'

export interface EmailSignatureOptions {
  HTMLAttributes: Record<string, any>;
  renderHTML: (() => string) | null;
}

export interface EmailSignatureAttributes {
  signatureId: string | null;
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    emailSignature: {
      insertEmailSignature: (
        attributes?: EmailSignatureAttributes,
      ) => ReturnType;
    };
  }
}

export const EmailSignature = Node.create<EmailSignatureOptions>({
  name: "emailSignature",
  group: "block",
  atom: true,
  draggable: true,
  selectable: true,
  inline: false,
  marks: "",
  isolating: true,
  content: "",

  addOptions() {
    return {
      HTMLAttributes: {},
      renderHTML: null,
      button: ({ editor, t }) => ({
        component: ActionButton,
        componentProps: {
          action: () => editor?.chain().focus().insertEmailSignature().run(),
          disabled: !editor?.isEditable,
          icon: 'signature',
          shortcutKeys: ['mod', 'Shift', 'S'],
          tooltip: t('composer.signature.tooltip'),
        },
      }),
    };
  },

  addAttributes() {
    return {
      signatureId: {
        default: null,
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="email-signature"]',
      },
    ];
  },

  addCommands() {
    return {
      insertEmailSignature:
        (attributes?: EmailSignatureAttributes) =>
        ({
          tr,
          dispatch,
          editor,
        }: {
          tr: any;
          dispatch: any;
          editor: Editor;
        }) => {
          if (!dispatch) {
            return false;
          }

          // Create the signature node with the given attributes
          const signatureNode =
            editor.schema.nodes[this.name].create(attributes);

          // Find any existing signature and remove it
          let signaturePos: number | null = null;
          tr.doc.descendants((node, pos) => {
            if (node.type.name === this.name) {
              signaturePos = pos;
              return false; // stop traversal
            }
            return true;
          });

          if (signaturePos !== null) {
            // Remove the existing signature
            tr.delete(signaturePos, signaturePos + 1);
          }

          // Always insert at the end of the document
          const endPosition = tr.doc.content.size;

          // Add a newline before the signature if needed
          const lastNode = tr.doc.lastChild;
          const needsNewLine =
            lastNode &&
            lastNode.type.name !== "paragraph" &&
            lastNode.type.name !== this.name;

          if (needsNewLine) {
            const paragraphNode = editor.schema.nodes.paragraph.create();
            tr.insert(endPosition, paragraphNode);
            tr.insert(endPosition + 1, signatureNode);
          } else {
            tr.insert(endPosition, signatureNode);
          }

          return true;
        },
    };
  },

  renderHTML({ HTMLAttributes }) {
    const { renderHTML } = this.options;

    if (typeof renderHTML === "function") {
      const wrapper = document.createElement("div");
      wrapper.setAttribute("data-type", "email-signature");

      Object.entries(HTMLAttributes).forEach(([key, value]) => {
        if (value !== null && value !== undefined) {
          wrapper.setAttribute(key, String(value));
        }
      });

      wrapper.innerHTML = renderHTML();

      return wrapper;
    }

    return [
      "div",
      mergeAttributes({ "data-type": "email-signature" }, HTMLAttributes),
      "",
    ];
  },
});
