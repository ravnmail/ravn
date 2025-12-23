import { invoke } from '@tauri-apps/api/core'
import { onMounted, ref } from 'vue'

export interface ThemeInfo {
  id: string
  name: string
  source: 'builtin' | 'user'
}

export function useTheme() {
  const themes = ref<ThemeInfo[]>([])
  const currentTheme = ref<string>('builtin/dark.css')
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  let themeStyleElement: HTMLStyleElement | null = null

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

  const getCurrentTheme = async () => {
    try {
      const themeId = await invoke<string>('get_current_theme')
      currentTheme.value = themeId
      return themeId
    } catch (err) {

      currentTheme.value = 'builtin/light.css'
      return 'builtin/light.css'
    }
  }

  const loadThemeCSS = (css: string) => {
    if (themeStyleElement) {
      themeStyleElement.remove()
    }

    themeStyleElement = document.createElement('style')
    themeStyleElement.setAttribute('data-theme-style', 'true')
    themeStyleElement.textContent = css
    document.head.appendChild(themeStyleElement)
  }

  const previewTheme = async (themeId: string) => {
    try {
      isLoading.value = true
      error.value = null
      const css = await invoke<string>('get_theme', { themeId })
      loadThemeCSS(css)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      throw err
    } finally {
      isLoading.value = false
    }
  }

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

  const initializeTheme = async () => {
    try {
      const themeId = await getCurrentTheme()
      const css = await invoke<string>('get_theme', { themeId })

      loadThemeCSS(css)

      await listThemes()
    } catch (err) {
      console.error('Failed to initialize theme:', err)
      try {
        const css = await invoke<string>('get_theme', { themeId: 'builtin/light.css' })
        loadThemeCSS(css)
      } catch (fallbackErr) {
        console.error('Failed to load fallback theme:', fallbackErr)
      }
    }
  }

  onMounted(() => {
    initializeTheme()
  })

  return {
    themes,
    currentTheme,
    isLoading,
    error,

    listThemes,
    getCurrentTheme,
    previewTheme,
    switchTheme,
    initializeTheme,
  }
}
