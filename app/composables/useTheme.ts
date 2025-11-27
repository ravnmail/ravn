import { invoke } from '@tauri-apps/api/core'
import { onMounted, ref } from 'vue'

export interface ThemeInfo {
  id: string
  name: string
  source: 'builtin' | 'user'
}

export function useTheme() {
  const themes = ref<ThemeInfo[]>([])
  const currentTheme = ref<string>('builtin/light.css')
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Dynamic style element for theme CSS
  let themeStyleElement: HTMLStyleElement | null = null

  /**
   * List all available themes
   */
  const listThemes = async () => {
    try {
      isLoading.value = true
      error.value = null
      const result = await invoke<ThemeInfo[]>('list_themes')
      themes.value = result
      return result
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Get the current theme from settings
   */
  const getCurrentTheme = async () => {
    try {
      const themeId = await invoke<string>('get_current_theme')
      currentTheme.value = themeId
      return themeId
    } catch (err) {
      // Default to light theme if not set
      currentTheme.value = 'builtin/light.css'
      return 'builtin/light.css'
    }
  }

  /**
   * Load theme CSS into the document
   */
  const loadThemeCSS = (css: string) => {
    // Remove existing theme style element if it exists
    if (themeStyleElement) {
      themeStyleElement.remove()
    }

    // Create new style element
    themeStyleElement = document.createElement('style')
    themeStyleElement.setAttribute('data-theme-style', 'true')
    themeStyleElement.textContent = css
    document.head.appendChild(themeStyleElement)
  }

  /**
   * Preview a theme without saving the preference
   */
  const previewTheme = async (themeId: string) => {
    try {
      isLoading.value = true
      error.value = null
      const css = await invoke<string>('get_theme', { themeId })
      loadThemeCSS(css)
      currentTheme.value = themeId
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Switch to a theme and save the preference
   */
  const switchTheme = async (themeId: string) => {
    try {
      isLoading.value = true
      error.value = null
      const css = await invoke<string>('switch_theme', { themeId })
      loadThemeCSS(css)
      currentTheme.value = themeId
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Initialize theme on mount
   */
  const initializeTheme = async () => {
    try {
      // Get current theme from settings
      const themeId = await getCurrentTheme()

      // Load the theme CSS
      const css = await invoke<string>('get_theme', { themeId })
      loadThemeCSS(css)

      // Load available themes list
      await listThemes()
    } catch (err) {
      console.error('Failed to initialize theme:', err)
      // Try to load default light theme as fallback
      try {
        const css = await invoke<string>('get_theme', { themeId: 'builtin/light.css' })
        loadThemeCSS(css)
      } catch (fallbackErr) {
        console.error('Failed to load fallback theme:', fallbackErr)
      }
    }
  }

  // Auto-initialize on mount
  onMounted(() => {
    initializeTheme()
  })

  return {
    // State
    themes,
    currentTheme,
    isLoading,
    error,

    // Methods
    listThemes,
    getCurrentTheme,
    previewTheme,
    switchTheme,
    initializeTheme,
  }
}
