import { getCurrentApp } from '../index.js'

setInterval(() => {
  console.log(getCurrentApp())
}, 1000)
