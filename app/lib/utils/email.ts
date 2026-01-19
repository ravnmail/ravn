import type { EmailAddress } from '~/types/email'

export function parseEmailAddress(value: string): EmailAddress {
  const emailRegex = /^(?:"?([^"<]*?)"?\s*)?<(.+?)>$|^(.+)$/
  const match = value.trim().match(emailRegex)

  if (!match) {
    throw new Error(`Invalid email address: ${value}`)
  }

  const name = match[1]?.trim()
  const address = (match[2] || match[3])?.trim()

  if (!address) {
    throw new Error(`Invalid email address: ${value}`)
  }

  return {
    address,
    ...(name && { name }),
  }
}
