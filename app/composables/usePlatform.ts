import { platform } from '@tauri-apps/plugin-os'
export type Platform = 'mac' | 'windows' | 'linux' | 'ios' | 'android'

export const usePlatform = () => {
  const currentPlatform = platform()

  const mapPlatform = (platformStr: string): Platform | null => {
    switch (platformStr.toLowerCase()) {
      case 'ios':
        return 'ios'
      case 'android':
        return 'android'
      case 'macos':
        return 'mac'
      case 'windows':
        return 'windows'
      case 'freebsd':
      case 'dragonfly':
      case 'netbsd':
      case 'openbsd':
      case 'solaris':
      case 'linux':
        return 'linux'
      default:
        return null
    }
  }

  const setPlatformClass = () => {
    const html = document.documentElement
    const platform = mapPlatform(currentPlatform)
    html.classList.remove('platform-mac', 'platform-windows', 'platform-linux')
    html.classList.add(`platform-${platform}`)
  }

  const getPlatform = (): Platform | null => {
    return mapPlatform(currentPlatform)
  }

  const initPlatform = () => {
    setPlatformClass()
  }

  return {
    getPlatform,
    setPlatformClass,
    initPlatform,
  }
}
