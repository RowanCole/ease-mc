import { LazyStore } from '@tauri-apps/plugin-store'

const STORE = new LazyStore('config.json')

const DEFAULTS = {
  gameDir: false,
}

export async function loadSettings() {
  const settings = {}
  for (const [key, fallback] of Object.entries(DEFAULTS)) {
    const val = await STORE.get(key)
    settings[key] = val !== undefined ? val : fallback
  }
  return settings
}

export async function saveSettings(settings) {
  for (const [key, val] of Object.entries(settings)) {
    await STORE.set(key, val)
  }
}
