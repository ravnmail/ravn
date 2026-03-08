import { invoke } from '@tauri-apps/api/core'
import { parseEmailAddress } from '~/lib/utils/email'
import type { EmailAddress } from '~/types/email'
import type { ComposerSeed } from './useComposerState'
import { useComposerState } from './useComposerState'

export interface RavnUrl {
  scheme: string
  path: string
  query?: string
}

/**
 * Parse an incoming navigation URL and return the frontend router path
 */
async function parseNavigationUrl(url: string): Promise<string> {
  try {
    console.log('[Navigation] Parsing navigation URL:', url)
    const routerPath = await invoke<string>('navigate_to_url', { url })
    console.log('[Navigation] Mapped to router path:', routerPath)
    return routerPath
  }
  catch (error) {
    console.error('[Navigation] Failed to parse navigation URL:', url, error)
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
  const { openComposer } = useComposerState()

  const handleInternalNavigation = async (target: string) => {
    if (target.startsWith('/compose')) {
      openComposer(parseComposeSeed(target))
      return
    }

    await router.push(target)
  }

  if (url.startsWith('ravn://') || url.startsWith('mailto:')) {
    const routerPath = await parseNavigationUrl(url)
    console.log('[Navigation] Navigating to:', routerPath)
    await handleInternalNavigation(routerPath)
  }
  else {
    console.log('[Navigation] Direct path navigation:', url)
    await handleInternalNavigation(url)
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

function parseComposeSeed(url: string): ComposerSeed {
  const params = new URLSearchParams(url.split('?')[1] || '')

  return {
    to: parseRecipients(params.getAll('to')),
    cc: parseRecipients(params.getAll('cc')),
    bcc: parseRecipients(params.getAll('bcc')),
    subject: params.get('subject') || '',
    body: params.get('body') || '',
  }
}

function parseRecipients(values: string[]): EmailAddress[] {
  return values
    .flatMap(value => value.split(','))
    .map(value => value.trim())
    .filter(Boolean)
    .map((value) => {
      try {
        return parseEmailAddress(value)
      }
      catch {
        return { address: value }
      }
    })
}
