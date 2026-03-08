import type { EmailAddress } from '~/types/email'

export interface ComposerSeed {
  to: EmailAddress[]
  cc: EmailAddress[]
  bcc: EmailAddress[]
  subject: string
  body: string
}

function createEmptySeed(): ComposerSeed {
  return {
    to: [],
    cc: [],
    bcc: [],
    subject: '',
    body: '',
  }
}

export function useComposerState() {
  const isOpen = useState<boolean>('composer:is-open', () => false)
  const sessionKey = useState<number>('composer:session-key', () => 0)
  const seed = useState<ComposerSeed>('composer:seed', createEmptySeed)

  const normalizeAddresses = (addresses?: EmailAddress[]) =>
    (addresses ?? []).map(address => ({ ...address }))

  const openComposer = (nextSeed?: Partial<ComposerSeed>) => {
    seed.value = {
      to: normalizeAddresses(nextSeed?.to),
      cc: normalizeAddresses(nextSeed?.cc),
      bcc: normalizeAddresses(nextSeed?.bcc),
      subject: nextSeed?.subject ?? '',
      body: nextSeed?.body ?? '',
    }
    sessionKey.value += 1
    isOpen.value = true
  }

  const closeComposer = () => {
    isOpen.value = false
    seed.value = createEmptySeed()
  }

  return {
    isOpen,
    sessionKey,
    seed,
    openComposer,
    closeComposer,
  }
}
