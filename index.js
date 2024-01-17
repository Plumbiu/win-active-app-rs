import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const { platform, arch } = process

function bind() {
  try {
    if (platform !== 'win32') {
      throw new Error('only support windows os')
    }
    if (arch === 'x64') {
      return require('./win-active-app-rs.win32-x64-msvc.node')
    }
    if (arch === 'ia32') {
      return require('./win-active-app-rs.win32-ia32-msvc.node')
    }
    if (arch === 'arm64') {
      return require('./win-active-app-rs.win32-arm64-msvc.node')
    } else {
      throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
  } catch (err) {
    console.log(err.message)
    return null
  }
}

const nativeBinding = bind()
if (!nativeBinding) {
  throw new Error('Failed to load native binding')
}

const { getCurrentApp, getAppIcon } = nativeBinding

export { getCurrentApp, getAppIcon }
