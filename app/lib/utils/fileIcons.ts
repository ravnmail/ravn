const mimeTypeToIcon: { [key: string]: string } = {
  // text / code
  'text/plain': 'markdown',
  'text/html': 'html',
  'text/css': 'css',
  'text/xml': 'xml',
  'text/x-python': 'python',
  'text/x-java': 'java',
  'text/x-csharp': 'csharp',
  'text/x-rust': 'rust',
  'text/x-typescript': 'typescript',
  'text/javascript': 'javascript',
  'text/x-ruby': 'ruby',
  'text/x-php': 'php',
  'text/x-go': 'go',
  'text/x-kotlin': 'kotlin',
  'text/x-swift': 'swift',
  'text/x-scala': 'scala',
  'text/x-lua': 'lua',
  'text/x-r': 'r',
  'text/x-elixir': 'elixir',
  'text/x-elm': 'elm',
  'text/markdown': 'markdown',
  'text/x-markdown': 'markdown',
  'text/x-tex': 'tex',
  'text/x-latex': 'latex',
  'text/x-handlebars': 'handlebars',
  'text/x-graphql': 'graphql',
  'text/x-dockerfile': 'docker',
  'text/x-makefile': 'makefile',
  'text/x-powershell': 'powershell',
  'text/x-shellscript': 'powershell',
  'text/x-diff': 'diff',
  'text/x-log': 'log',
  // structured
  'application/json': 'json',
  'application/xml': 'xml',
  'application/yaml': 'yaml',
  'application/x-yaml': 'yaml',
  'application/toml': 'toml',
  'application/sql': 'markdown',
  'application/graphql': 'graphql',
  'application/x-httpd-vue': 'vue',
  // images / media
  'image/svg+xml': 'svg',
  'image/png': 'image',
  'image/jpeg': 'image',
  'image/jpg': 'image',
  'image/gif': 'image',
  'image/webp': 'image',
  'image/avif': 'image',
  'image/bmp': 'image',
  'image/x-icon': 'image',
  'video/mp4': 'video',
  'video/quicktime': 'video',
  'video/webm': 'video',
  'audio/mpeg': 'audio',
  'audio/wav': 'audio',
  'audio/ogg': 'audio',
  // archives / executables / fonts
  'application/pdf': 'pdf',
  'application/zip': 'zip',
  'application/x-7z-compressed': 'zip',
  'application/x-rar-compressed': 'zip',
  'application/gzip': 'zip',
  'application/x-tar': 'zip',
  'application/java-archive': 'jar',
  'application/x-msdownload': 'exe',
  'application/x-httpd-php': 'php',
  // office
  'application/vnd.openxmlformats-officedocument.wordprocessingml.document': 'word',
  'application/msword': 'word',
  'application/vnd.ms-excel': 'table',
  'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet': 'table',
  'application/vnd.openxmlformats-officedocument.presentationml.presentation': 'powerpoint',
  'application/vnd.adobe.photoshop': 'adobe-photoshop',
  'application/epub+zip': 'epub',
  // fonts / certificates
  'font/ttf': 'font',
  'font/otf': 'font',
  'font/woff': 'font',
  'font/woff2': 'font',
  'application/x-font-ttf': 'font',
  'application/x-font-woff': 'font',
  'application/pkix-cert': 'certificate',
  'application/x-x509-ca-cert': 'certificate'
}

const extensionToIcon: { [ext: string]: string } = {
  // 3d
  'glb': '3d', 'gltf': '3d', 'stl': '3d', 'obj': '3d',
  // languages
  'ada': 'ada', 'adb': 'ada', 'ads': 'ada',
  'asm': 'assembly', 's': 'assembly',
  'c': 'csharp', 'h': 'csharp',
  'cpp': 'objective-cpp', 'cc': 'objective-cpp', 'cxx': 'objective-cpp',
  'cs': 'csharp',
  'java': 'java', 'kt': 'kotlin', 'kts': 'kotlin',
  'py': 'python', 'pyi': 'python',
  'rs': 'rust',
  'go': 'go',
  'dart': 'dart',
  'scala': 'scala',
  'swift': 'swift',
  'lua': 'lua',
  'rb': 'ruby',
  'r': 'r',
  'ex': 'elixir', 'exs': 'elixir',
  'elm': 'elm',
  'hs': 'haskell',
  'm': 'objective-c', 'mm': 'objective-cpp',
  'ts': 'typescript', 'tsx': 'typescript',
  'js': 'javascript', 'jsx': 'javascript', 'mjs': 'javascript', 'cjs': 'javascript',
  'php': 'php',
  'sql': 'markdown',
  // markup / styles
  'html': 'html', 'htm': 'html',
  'css': 'css',
  'json': 'json', 'jsonl': 'json',
  'md': 'markdown', 'markdown': 'markdown', 'mdown': 'markdown',
  'xml': 'xml',
  'yaml': 'yaml', 'yml': 'yaml',
  'toml': 'toml',
  'tex': 'tex',
  'bib': 'bibtex-style',
  'bibtex': 'bibtex-style',
  'handlebars': 'handlebars', 'hbs': 'handlebars',
  'graphql': 'graphql', 'gql': 'graphql',
  // styles / preprocessors
  'scss': 'sass', 'sass': 'sass', 'less': 'css',
  // images / media
  'png': 'image', 'jpg': 'image', 'jpeg': 'image', 'gif': 'image', 'webp': 'image', 'avif': 'image',
  'bmp': 'image', 'ico': 'image', 'svg': 'svg',
  'mp4': 'video', 'mov': 'video', 'mkv': 'video', 'avi': 'video', 'webm': 'video',
  'mp3': 'audio', 'wav': 'audio', 'ogg': 'audio', 'flac': 'audio', 'm4a': 'audio',
  // documents / office
  'pdf': 'pdf',
  'doc': 'word', 'docx': 'word',
  'xls': 'table', 'xlsx': 'table', 'csv': 'table',
  'ppt': 'powerpoint', 'pptx': 'powerpoint',
  'epub': 'epub',
  // archives / executables / libs
  'zip': 'zip', 'rar': 'zip', '7z': 'zip', 'tar': 'zip', 'gz': 'zip', 'tgz': 'zip', 'bz2': 'zip',
  'exe': 'exe', 'msi': 'exe',
  'dll': 'dll', 'so': 'dll', 'dylib': 'dll',
  'jar': 'jar',
  // fonts
  'ttf': 'font', 'otf': 'font', 'woff': 'font', 'woff2': 'font',
  // images/editing / design
  'psd': 'adobe-photoshop', 'ai': 'adobe-illustrator', 'sketch': 'sketch', 'fig': 'figma',
  'blend': 'blender',
  // misc
  'lock': 'json', 'env': 'http', 'ini': 'key', 'pem': 'certificate', 'crt': 'certificate', 'key': 'certificate',
  'log': 'log', 'diff': 'diff', 'patch': 'diff',
  'makefile': 'makefile', 'mk': 'makefile', 'gradle': 'java',
  'dockerfile': 'docker',
  // frameworks / configs
  'vue': 'vue', 'svelte': 'vue',
  'yarn': 'javascript', 'npmignore': 'javascript',
  // notebooks / special
  'ipynb': 'python',
  'pt': 'pytorch', 'pth': 'pytorch', 'ckpt': 'pytorch',
  // art / fonts / lottie
  'lottie': 'lottie', 'jsonlottie': 'lottie',
  // other language-specific / tooling
  'cmake': 'makefile',
  'rbxl': 'roblox', 'mcfunction': 'minecraft',
  'uml': 'uml', 'swagger': 'swagger',
  'zig': 'zig'
}

const specialFilenames: { [name: string]: string } = {
  'dockerfile': 'docker',
  'makefile': 'makefile',
  'readme': 'markdown',
  'readme.md': 'markdown',
  'license': 'certificate',
  'package.json': 'json',
  'package-lock.json': 'json',
  'yarn.lock': 'json',
  'gemfile': 'ruby',
  'rakefile': 'ruby',
  'cargo.toml': 'toml',
  'podfile': 'ruby',
  'vagrantfile': 'url'
}

function lookupByExtension(filename: string): string | undefined {
  if (!filename) return undefined
  const name = filename.toLowerCase()
  // exact special name first
  if (specialFilenames[name]) return specialFilenames[name]
  // split by dot
  const parts = name.split('.').filter(Boolean)
  if (parts.length === 0) return undefined
  // try multi-extension (e.g. tar.gz)
  if (parts.length >= 2) {
    const lastTwo = parts.slice(-2).join('.')
    if (extensionToIcon[lastTwo]) return extensionToIcon[lastTwo]
    // aside: common tar.gz -> treat as zip
    if (lastTwo === 'tar.gz' || lastTwo === 'tar.bz2' || lastTwo === 'tar.xz') return 'zip'
  }
  const ext = parts[parts.length - 1]
  return extensionToIcon[ext] ?? undefined
}

export function getFileIconForMimeType(mimeType: string, filename: string): string {
  const normalizedMime = (mimeType || '').split(';')[0].trim().toLowerCase()
  if (normalizedMime && mimeTypeToIcon[normalizedMime]) {
    return `filetype:${mimeTypeToIcon[normalizedMime]}`
  }

  const basename = (filename || '').split(/[\\/]/).pop() || ''
  const fileIcon = lookupByExtension(basename)
  if (fileIcon) return `filetype:${fileIcon}`

  return `filetype:assembly` // default icon
}