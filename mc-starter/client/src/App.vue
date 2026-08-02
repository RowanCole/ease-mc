<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// === State ===
const status = ref('ready') // ready | playing
const showSettings = ref(false)
const showNews = ref(false)

// Game info
const gameInfo = {
  name: '游戏名称',
  version: '1.0.0',
  description: 'welcome to Minecraft'
}

// Settings
const settings = ref({
  ram: 4096,
  gameDir: '%appdata%\\.minecraft',
  autoClose: true,
})

const statusText = ref('就绪')

// === Actions ===
async function startGame() {
  if (status.value === 'playing') {
    try {
      await invoke('close_game')
      status.value = 'ready'
      statusText.value = '就绪'
    } catch (e) {
      console.error('Close failed:', e)
    }
    return
  }

  if (status.value !== 'ready') return

  try {
    await invoke('launch_game')
    status.value = 'playing'
    statusText.value = '游戏运行中'
  } catch (e) {
    console.error('Launch failed:', e)
  }
}
</script>

<template>
  <div class="app">
    <!-- === Animated Background === -->
    <div class="bg-layer">
      <div class="bg-gradient"></div>
      <div class="bg-grid"></div>
    </div>

    <!-- === Top Bar === -->
    <div class="top-bar">
      <div class="top-left">
        <div class="game-badge">{{ gameInfo.version }}</div>
      </div>
      <div class="top-right">
        <button class="top-btn" @click="showNews = !showNews" title="公告">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
            <path d="M13.73 21a2 2 0 0 1-3.46 0" />
          </svg>
        </button>
        <button class="top-btn" @click="showSettings = !showSettings" title="设置">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
          </svg>
        </button>
      </div>
    </div>

    <!-- === Center Content === -->
    <main class="center">
      <!-- Logo Area -->
      <div class="logo-area">
        <div class="logo-icon">
          <svg viewBox="0 0 80 80" width="80" height="80" fill="none">
            <defs>
              <linearGradient id="lg" x1="0" y1="0" x2="80" y2="80">
                <stop offset="0%" stop-color="#4FC3F7" />
                <stop offset="100%" stop-color="#0288D1" />
              </linearGradient>
            </defs>
            <rect x="8" y="8" width="64" height="64" rx="16" fill="url(#lg)" />
            <path d="M28 40l10 10 16-16" stroke="#fff" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </div>
        <h1 class="game-title">{{ gameInfo.name }}</h1>
        <p class="game-desc">{{ gameInfo.description }}</p>
      </div>

      <!-- Launch Section -->
      <div class="launch-area">
        <button
          class="play-btn"
          :class="{ active: status === 'playing' }"
          @click="startGame"
        >
          <span class="btn-ring"></span>
          <span class="btn-content">
            <svg v-if="status === 'ready'" viewBox="0 0 24 24" width="22" height="22" fill="currentColor">
              <path d="M8 5v14l11-7z" />
            </svg>
            <svg v-else viewBox="0 0 24 24" width="22" height="22" fill="currentColor">
              <rect x="6" y="4" width="4" height="16" rx="1" />
              <rect x="14" y="4" width="4" height="16" rx="1" />
            </svg>
            <span>{{ status === 'ready' ? '开始游戏' : '退出' }}</span>
          </span>
        </button>
      </div>
    </main>

    <!-- === Bottom Bar === -->
    <div class="bottom-bar">
      <span class="copyright">© {{ new Date().getFullYear() }} {{ gameInfo.name }}</span>
      <span class="version">{{ gameInfo.version }} · {{ statusText }}</span>
    </div>

    <!-- === News Panel === -->
    <Teleport to="body">
      <Transition name="panel">
        <div v-if="showNews" class="side-panel">
          <div class="panel-header">
            <h2>公告</h2>
            <button class="top-btn" @click="showNews = false">
              <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>
          <div class="panel-body">
            <div class="news-card">
              <div class="news-date">2026-07-18</div>
              <div class="news-title">客户端 v1.0.0 发布</div>
              <div class="news-desc">首个公开版本，欢迎体验</div>
            </div>
            <div class="news-card">
              <div class="news-date">2026-07-10</div>
              <div class="news-title">内测招募开启</div>
              <div class="news-desc">限量内测资格发放中</div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- === Settings Panel === -->
    <Teleport to="body">
      <Transition name="panel">
        <div v-if="showSettings" class="side-panel">
          <div class="panel-header">
            <h2>设置</h2>
            <button class="top-btn" @click="showSettings = false">
              <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>
          <div class="panel-body">
            <div class="setting-item">
              <label>最大内存</label>
              <div class="setting-row">
                <input type="range" min="1024" max="16384" step="512" v-model.number="settings.ram" />
                <span class="setting-value">{{ settings.ram >= 1024 ? `${settings.ram / 1024} GB` : `${settings.ram} MB` }}</span>
              </div>
            </div>
            <div class="setting-item">
              <label>游戏目录</label>
              <div class="setting-dir">
                <span class="dir-text">{{ settings.gameDir }}</span>
              </div>
            </div>
            <div class="setting-item">
              <label class="toggle-row">
                <input type="checkbox" v-model="settings.autoClose" />
                <span class="toggle-track">
                  <span class="toggle-thumb"></span>
                </span>
                <span>启动后关闭启动器</span>
              </label>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style>
/* ===== Reset ===== */
*, *::before, *::after {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --accent: #67d5ff;
  --accent2: #229bd2;
  --bg-dark: #071019;
  --bg-mid: #0d1a26;
  --surface: rgba(12, 28, 42, 0.84);
  --surface-raised: rgba(16, 35, 51, 0.94);
  --text: #f4f8fb;
  --text-dim: #a8b8c5;
  --text-muted: #718392;
  --border: rgba(205, 230, 242, 0.12);
  --border-strong: rgba(205, 230, 242, 0.22);
  --radius-sm: 10px;
  --radius-md: 16px;
}

html {
  height: 100vh;
  overflow: hidden;
  background-color: var(--bg-dark);
  background-image: url('/src/assets/background.jpg');
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  color: var(--text);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Noto Sans SC', sans-serif;
  -webkit-font-smoothing: antialiased;
  user-select: none;
}

body {
  height: 100vh;
  overflow: hidden;
  background: transparent;
}

button,
input {
  font: inherit;
}

button {
  -webkit-tap-highlight-color: transparent;
}

#app {
  height: 100vh;
}
</style>

<style scoped>
/* ===== App Shell ===== */
.app {
  position: relative;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ===== Background ===== */
.bg-layer {
  position: absolute;
  inset: 0;
  z-index: 0;
  overflow: hidden;
}

.bg-gradient {
  position: absolute;
  inset: 0;
  background:
    linear-gradient(180deg, rgba(4, 12, 20, 0.48) 0%, rgba(4, 12, 20, 0.76) 58%, rgba(3, 9, 15, 0.92) 100%),
    linear-gradient(90deg, rgba(3, 10, 17, 0.5), transparent 42%, rgba(3, 10, 17, 0.35));
}

.bg-layer::after {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  box-shadow: inset 0 0 140px rgba(2, 8, 13, 0.62);
}

.bg-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(255,255,255,0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255,255,255,0.015) 1px, transparent 1px);
  background-size: 56px 56px;
  opacity: 0.5;
  mask-image: linear-gradient(180deg, transparent, black 18%, black 78%, transparent);
  -webkit-mask-image: linear-gradient(180deg, transparent, black 18%, black 78%, transparent);
}

/* ===== Top Bar ===== */
.top-bar {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 68px;
  padding: 16px 22px;
  border-bottom: 1px solid rgba(205, 230, 242, 0.08);
  background: linear-gradient(180deg, rgba(5, 15, 24, 0.48), rgba(5, 15, 24, 0.08));
  -webkit-app-region: drag;
}

.top-left,
.top-right {
  display: flex;
  align-items: center;
  gap: 6px;
  -webkit-app-region: no-drag;
}

.game-badge {
  display: inline-flex;
  align-items: center;
  min-height: 26px;
  font-size: 11px;
  color: var(--accent);
  background: rgba(103, 213, 255, 0.1);
  border: 1px solid rgba(103, 213, 255, 0.24);
  padding: 2px 11px;
  border-radius: 20px;
  font-weight: 500;
  letter-spacing: 0.4px;
}

.top-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  padding: 0;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: rgba(8, 23, 35, 0.42);
  color: var(--text-dim);
  cursor: pointer;
  transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease, transform 0.2s ease;
  -webkit-app-region: no-drag;
}

.top-btn:hover {
  background: rgba(103, 213, 255, 0.1);
  border-color: rgba(103, 213, 255, 0.22);
  color: var(--text);
  transform: translateY(-1px);
}

.top-btn:focus-visible,
.play-btn:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 3px;
}

/* ===== Center ===== */
.center {
  position: relative;
  z-index: 5;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 28px;
  padding: 32px;
}

/* ===== Logo Area ===== */
.logo-area {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 13px;
}

.logo-icon {
  display: flex;
  width: 84px;
  height: 84px;
  align-items: center;
  justify-content: center;
  filter: drop-shadow(0 14px 28px rgba(0, 0, 0, 0.34));
  animation: logo-breathe 8s ease-in-out infinite;
}

.game-title {
  max-width: min(100%, 560px);
  color: var(--text);
  font-size: 36px;
  line-height: 1.15;
  font-weight: 700;
  letter-spacing: 1px;
  background: linear-gradient(135deg, #fff 60%, var(--accent));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.game-desc {
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-dim);
  letter-spacing: 0.5px;
}

@keyframes logo-breathe {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-3px); }
}

/* ===== Launch Area ===== */
.launch-area {
  width: min(100%, 320px);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

/* Play Button */
.play-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-height: 58px;
  border: none;
  border-radius: 14px;
  cursor: pointer;
  background: linear-gradient(135deg, #6dd9ff, #258fc6);
  color: #fff;
  overflow: hidden;
  transition: transform 0.2s ease, box-shadow 0.2s ease, filter 0.2s ease;
  box-shadow: 0 12px 28px rgba(8, 99, 143, 0.28), inset 0 1px 0 rgba(255, 255, 255, 0.26);
}

.play-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  filter: brightness(1.05);
  box-shadow: 0 16px 34px rgba(8, 99, 143, 0.36), inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.play-btn:active {
  transform: translateY(0);
  filter: brightness(0.98);
}

.play-btn.active {
  background: linear-gradient(135deg, #2876bc, #18538a);
  box-shadow: 0 12px 28px rgba(10, 62, 111, 0.34), inset 0 1px 0 rgba(255, 255, 255, 0.18);
}

.btn-ring {
  position: absolute;
  inset: 1px;
  border: 1px solid rgba(255, 255, 255, 0.25);
  border-radius: 13px;
  opacity: 0.65;
  pointer-events: none;
}

.btn-content {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
  z-index: 1;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.8px;
}

/* ===== Bottom Bar ===== */
.bottom-bar {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 50px;
  gap: 16px;
  padding: 12px 22px 16px;
  border-top: 1px solid rgba(205, 230, 242, 0.08);
  background: linear-gradient(180deg, rgba(5, 15, 24, 0.04), rgba(5, 15, 24, 0.38));
  font-size: 11px;
  color: var(--text-muted);
}

.copyright,
.version {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ===== Side Panel ===== */
.side-panel {
  position: fixed;
  top: 0;
  right: 0;
  width: 340px;
  max-width: 100vw;
  height: 100vh;
  background: var(--surface-raised);
  backdrop-filter: blur(18px) saturate(120%);
  border-left: 1px solid var(--border);
  box-shadow: -24px 0 54px rgba(1, 7, 12, 0.36);
  z-index: 1000;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 68px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.panel-header h2 {
  color: var(--text);
  font-size: 16px;
  font-weight: 600;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 18px 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* News */
.news-card {
  padding: 15px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  box-shadow: 0 8px 20px rgba(1, 8, 13, 0.14);
}

.news-date {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.news-title {
  font-size: 14px;
  line-height: 1.4;
  font-weight: 600;
  margin-bottom: 4px;
}

.news-desc {
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-dim);
}

/* Settings */
.setting-item {
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.setting-item label {
  font-size: 12px;
  color: var(--text-dim);
  font-weight: 500;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.setting-row input[type="range"] {
  flex: 1;
  -webkit-appearance: none;
  height: 4px;
  background: rgba(205, 230, 242, 0.14);
  border-radius: 2px;
  outline: none;
}

.setting-row input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  box-shadow: 0 0 0 4px rgba(103, 213, 255, 0.12), 0 0 10px rgba(103, 213, 255, 0.34);
}

.setting-row input[type="range"]:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 5px;
}

.setting-value {
  font-size: 13px;
  font-weight: 600;
  min-width: 50px;
  text-align: right;
}

.setting-dir {
  background: rgba(7, 20, 31, 0.62);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 10px 12px;
}

.dir-text {
  font-size: 12px;
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  color: var(--text-muted);
  word-break: break-all;
}

.toggle-row {
  display: flex !important;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  flex-direction: row !important;
}

.toggle-row input {
  display: none;
}

.toggle-track {
  position: relative;
  width: 36px;
  height: 20px;
  background: rgba(205, 230, 242, 0.14);
  border-radius: 10px;
  transition: background-color 0.2s ease;
  flex-shrink: 0;
}

.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  background: var(--text-muted);
  border-radius: 50%;
  transition: left 0.2s ease, background-color 0.2s ease;
}

.toggle-row input:checked + .toggle-track {
  background: var(--accent);
}

.toggle-row input:checked + .toggle-track .toggle-thumb {
  left: 18px;
  background: #fff;
}

/* Panel Transition */
.panel-enter-active {
  transition: transform 0.24s ease;
}

.panel-leave-active {
  transition: transform 0.2s ease;
}

.panel-enter-from,
.panel-leave-to {
  transform: translateX(100%);
}

@media (max-width: 560px) {
  .top-bar {
    min-height: 60px;
    padding: 13px 16px;
  }

  .center {
    gap: 24px;
    padding: 24px 20px 18px;
  }

  .logo-icon {
    width: 70px;
    height: 70px;
  }

  .logo-icon svg {
    width: 70px;
    height: 70px;
  }

  .game-title {
    max-width: calc(100vw - 40px);
    font-size: 30px;
  }

  .game-desc {
    max-width: calc(100vw - 40px);
    font-size: 12px;
  }

  .bottom-bar {
    min-height: 58px;
    align-items: flex-start;
    flex-direction: column;
    gap: 4px;
    padding: 10px 16px 14px;
  }

  .side-panel {
    width: 100vw;
  }

  .panel-header {
    min-height: 60px;
    padding: 14px 16px;
  }

  .panel-body {
    padding: 16px 16px 22px;
  }
}

@media (max-height: 520px) {
  .center {
    gap: 18px;
    padding-top: 16px;
    padding-bottom: 12px;
  }

  .logo-icon {
    width: 64px;
    height: 64px;
  }

  .logo-icon svg {
    width: 64px;
    height: 64px;
  }

  .game-title {
    font-size: 30px;
  }

  .play-btn {
    min-height: 52px;
  }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    scroll-behavior: auto !important;
    transition-duration: 0.01ms !important;
  }
}
</style>
