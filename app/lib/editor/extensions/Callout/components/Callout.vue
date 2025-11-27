<!-- CalloutComponent.vue -->
<template>
  <node-view-wrapper class="callout-wrapper">
    <div
      :class="['callout', `callout-${node.attrs.type}`]"
      :data-callout-type="node.attrs.type"
      :data-emoji="node.attrs.emoji"
      :data-type="'callout'"
      @click.right.prevent="openTypeSelector"
    >
      <div
        class="callout-emoji"
        @click="openEmojiPicker"
      >
        {{ node.attrs.emoji }}
      </div>
      <div class="callout-content">
        <node-view-content class="callout-content-editor"/>
      </div>

      <div
        v-if="showEmojiPicker"
        class="emoji-picker"
      >
        <button
          v-for="emoji in commonEmojis"
          :key="emoji"
          class="emoji-option"
          @click="selectEmoji(emoji)"
        >
          {{ emoji }}
        </button>
        <input
          v-model="customEmoji"
          class="custom-emoji-input"
          placeholder="Type emoji..."
          @keyup.enter="selectEmoji(customEmoji)"
        >
      </div>

      <!-- Type selector -->
      <div
        v-if="showTypeSelector"
        class="type-selector"
      >
        <button
          v-for="type in calloutTypes"
          :key="type.value"
          class="type-option"
          @click="selectType(type.value)"
        >
          {{ type.label }}
        </button>
      </div>
    </div>
  </node-view-wrapper>
</template>

<script lang="ts">
import type { Editor, Node } from '@tiptap/core'
import { NodeViewWrapper, NodeViewContent } from '@tiptap/vue-3'
import type { CalloutType } from '../index'

interface CalloutTypeOption {
  value: CalloutType
  label: string
}

export default defineComponent({
  components: {
    NodeViewWrapper,
    NodeViewContent,
  },

  props: {
    editor: {
      type: Object as PropType<Editor>,
      required: true,
    },
    node: {
      type: Object as PropType<Node>,
      required: true,
    },
    updateAttributes: {
      type: Function as PropType<(attrs: Record<string, any>) => void>,
      required: true,
    },
    extension: {
      type: Object,
      required: true,
    },
  },

  data() {
    return {
      showEmojiPicker: false,
      showTypeSelector: false,
      customEmoji: '',
      commonEmojis: ['💡', '📝', '⚠️', '❗', '📌', '🔍', '💭', '❓', '🎯', '🚩'],
      calloutTypes: [
        { value: 'info' as const, label: 'Info' },
        { value: 'success' as const, label: 'Success' },
        { value: 'warning' as const, label: 'Warning' },
        { value: 'error' as const, label: 'Error' },
        { value: 'note' as const, label: 'Note' },
      ] as CalloutTypeOption[]
    }
  },

  methods: {
    openEmojiPicker(): void {
      this.showEmojiPicker = !this.showEmojiPicker
      this.showTypeSelector = false
    },

    selectEmoji(emoji: string): void {
      this.showEmojiPicker = false
      this.editor.commands.updateCalloutEmoji(emoji)
    },

    openTypeSelector(): void {
      this.showTypeSelector = !this.showTypeSelector
      this.showEmojiPicker = false
    },

    selectType(type: CalloutType): void {
      this.showTypeSelector = false
      this.editor.commands.updateCalloutType(type)
    }
  },
})
</script>

<style>
.callout {
  position: relative;
  display: flex;
  padding: 1rem;
  margin: 1rem 0;
  border-radius: 4px;
  background-color: #f5f5f5;
  min-height: 2rem;
}

.callout-emoji {
  margin-right: 0.75rem;
  cursor: pointer;
  user-select: none;
  min-width: 1.5rem;
  text-align: center;
}

.callout-content {
  flex: 1;
  min-width: 0;
}

.callout-content-editor {
  width: 100%;
}

/* Minimal styling for different callout types */
.callout-info {
  background-color: rgba(224, 242, 254, 0.5);
  border-left: 4px solid #0ea5e9;
}

.callout-success {
  background-color: rgba(220, 252, 231, 0.5);
  border-left: 4px solid #22c55e;
}

.callout-warning {
  background-color: rgba(254, 243, 199, 0.5);
  border-left: 4px solid #eab308;
}

.callout-error {
  background-color: rgba(254, 226, 226, 0.5);
  border-left: 4px solid #ef4444;
}

.callout-note {
  background-color: rgba(228, 228, 231, 0.5);
  border-left: 4px solid #71717a;
}

/* Emoji picker */
.emoji-picker {
  position: absolute;
  top: 100%;
  left: 0;
  display: flex;
  flex-wrap: wrap;
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  padding: 0.5rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  z-index: 10;
  max-width: 200px;
}

.emoji-option {
  padding: 0.25rem;
  cursor: pointer;
  border: none;
  background: none;
  margin: 0.25rem;
}

.custom-emoji-input {
  width: 100%;
  padding: 0.25rem;
  margin-top: 0.5rem;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
}

/* Type selector */
.type-selector {
  position: absolute;
  top: 100%;
  left: 0;
  display: flex;
  flex-direction: column;
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  padding: 0.25rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  z-index: 10;
}

.type-option {
  padding: 0.25rem 0.5rem;
  cursor: pointer;
  border: none;
  background: none;
  text-align: left;
}

.type-option:hover {
  background-color: #f7fafc;
}
</style>