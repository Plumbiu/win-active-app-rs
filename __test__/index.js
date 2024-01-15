import { getCurrentAppPath, getCachedApps } from '../index.js'

console.log(getCachedApps())

const appPath = getCurrentAppPath()
console.log(appPath, appPath.length)
