// @ts-check
import withNuxt from './.nuxt/eslint.config.mjs'

export default withNuxt({
  settings: {
    tailwindcss: {
      whitelist: []
    },
  },
  rules: {
    "@typescript-eslint/no-unused-vars": ["error", {
      "varsIgnorePattern": "^_",
      "argsIgnorePattern": "^_",
      "caughtErrorsIgnorePattern": "^_"
    }],
    'no-console': 'off',
    'no-debugger': 'warn',
    'vue/multi-word-component-names': 'off',
    'vue/no-multiple-template-root': 'off',
    'vue/no-v-html': 'off',
    'vue/max-attributes-per-line': ['error', {
      singleline: {
        max: 1,
      },
      multiline: {
        max: 1,
      },
    }],
  },
})
