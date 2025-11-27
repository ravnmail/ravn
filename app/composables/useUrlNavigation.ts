import { invoke } from '@tauri-apps/api/core'

export interface RavnUrl {
  scheme: string
  path: string
  query?: string
}

/**
 * Parse a RAVN URL and return the frontend router path
 */
async function parseRavnUrl(url: string): Promise<string> {
  try {
    console.log('[Navigation] Parsing RAVN URL:', url)
    const routerPath = await invoke<string>('navigate_to_url', { url })
    console.log('[Navigation] Mapped to router path:', routerPath)
    return routerPath
  }
  catch (error) {
    console.error('[Navigation] Failed to parse RAVN URL:', url, error)
    return '/'
  }
}

/**
 * Build a RAVN URL from path and query components
 */
async function buildRavnUrl(path: string, query?: string): Promise<string> {
  try {
    const ravnUrl = await invoke<string>('build_ravn_url', { path, query: query || null })
    return ravnUrl
  }
  catch (error) {
    console.error('[Navigation] Failed to build RAVN URL:', error)
    return `ravn://${path}`
  }
}

/**
 * Navigate to a RAVN URL or plain path
 * Accepts: 'ravn://settings', '/settings/ai', or plain paths
 */
export async function navigateToUrl(url: string) {
  const router = useRouter()

  // If it's a ravn:// URL, parse it first
  if (url.startsWith('ravn://')) {
    const routerPath = await parseRavnUrl(url)
    console.log('[Navigation] Navigating to:', routerPath)
    router.push(routerPath)
  }
  // If it's already a path, navigate directly
  else {
    console.log('[Navigation] Direct path navigation:', url)
    router.push(url)
  }
}

export async function navigateToPath(path: string, query?: string) {
  const ravnUrl = await buildRavnUrl(path, query)
  await navigateToUrl(ravnUrl)
}

export const navigate = {
  toSettings: () => navigateToUrl('ravn://settings'),
  toSettingsAi: () => navigateToUrl('ravn://settings/ai'),
  toSettingsSignatures: () => navigateToUrl('ravn://settings/signatures'),
  toMailInbox: () => navigateToUrl('ravn://mail/inbox'),
  toMailSent: () => navigateToUrl('ravn://mail/sent'),
  toMailDrafts: () => navigateToUrl('ravn://mail/drafts'),
  toMailStarred: () => navigateToUrl('ravn://mail/starred'),
  toCompose: (to?: string) => {
    const url = to ? `ravn://compose?to=${encodeURIComponent(to)}` : 'ravn://compose'
    return navigateToUrl(url)
  },
  toAccount: (accountId: string) => navigateToUrl(`ravn://accounts/${accountId}`),
  toAccountFolder: (accountId: string, folderId: string) =>
    navigateToUrl(`ravn://accounts/${accountId}/folders/${folderId}`),
  toEmail: (emailId: string) => navigateToUrl(`ravn://mail/${emailId}`),
}
