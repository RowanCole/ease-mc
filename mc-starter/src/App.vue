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
      <div class="bg-orbs">
        <div class="orb o1"></div>
        <div class="orb o2"></div>
        <div class="orb o3"></div>
      </div>
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
  --accent: #4FC3F7;
  --accent2: #0288D1;
  --bg-dark: #0a0e17;
  --bg-mid: #111827;
  --text: #040404;
  --text-dim: #020202;
  --border: rgba(255,255,255,0.06);
  --radius: 12px;
}

  html {
    background-image: url('/src/assets/background.jpg');
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
    height: 100vh;
    overflow: hidden;
    color: var(--text);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Noto Sans SC', sans-serif;
    -webkit-font-smoothing: antialiased;
    user-select: none;
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
  background: rgba(0, 0, 0, 0.5);
}

.bg-orbs {
  position: absolute;
  inset: 0;
}

.orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.15;
}

.o1 {
  width: 600px;
  height: 600px;
  background: #4FC3F7;
  top: -200px;
  right: -150px;
  animation: drift 20s ease-in-out infinite;
}

.o2 {
  width: 400px;
  height: 400px;
  background: #0288D1;
  bottom: -100px;
  left: -100px;
  animation: drift 25s ease-in-out infinite reverse;
}

.o3 {
  width: 300px;
  height: 300px;
  background: #7C4DFF;
  top: 40%;
  left: 55%;
  animation: drift 18s ease-in-out infinite 5s;
}

@keyframes drift {
  0%, 100% { transform: translate(0, 0) scale(1); }
  33% { transform: translate(30px, -30px) scale(1.05); }
  66% { transform: translate(-20px, 15px) scale(0.95); }
}

.bg-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(255,255,255,0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255,255,255,0.015) 1px, transparent 1px);
  background-size: 48px 48px;
  mask-image: radial-gradient(ellipse 70% 60% at 50% 50%, black 30%, transparent 70%);
  -webkit-mask-image: radial-gradient(ellipse 70% 60% at 50% 50%, black 30%, transparent 70%);
}

/* ===== Top Bar ===== */
.top-bar {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  -webkit-app-region: drag;
}

.top-left,
.top-right {
  display: flex;
  align-items: center;
  gap: 10px;
  -webkit-app-region: no-drag;
}

.game-badge {
  font-size: 11px;
  color: var(--accent);
  background: rgba(79, 195, 247, 0.1);
  border: 1px solid rgba(79, 195, 247, 0.2);
  padding: 2px 10px;
  border-radius: 20px;
  font-weight: 500;
  letter-spacing: 0.3px;
}

.top-btn {
  background: none;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 6px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  -webkit-app-region: no-drag;
}

.top-btn:hover {
  background: rgba(255,255,255,0.06);
  color: var(--text);
}

/* ===== Center ===== */
.center {
  position: relative;
  z-index: 5;
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 32px;
  padding: 0 32px;
}

/* ===== Logo Area ===== */
.logo-area {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  margin-bottom: 4px;
  filter: drop-shadow(0 0 20px rgba(79, 195, 247, 0.3));
  animation: logo-float 4s ease-in-out infinite;
}

@keyframes logo-float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-6px); }
}

.game-title {
  font-size: 36px;
  font-weight: 700;
  letter-spacing: 2px;
  background: linear-gradient(135deg, #fff 60%, var(--accent));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.game-desc {
  font-size: 13px;
  color: var(--text-dim);
  letter-spacing: 0.5px;
}

/* ===== Launch Area ===== */
.launch-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  min-width: 280px;
}

/* Play Button */
.play-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 240px;
  height: 56px;
  border: none;
  border-radius: 28px;
  cursor: pointer;
  background: linear-gradient(135deg, var(--accent), var(--accent2));
  color: #fff;
  overflow: hidden;
  transition: all 0.3s;
  box-shadow: 0 4px 24px rgba(79, 195, 247, 0.3);
}

.play-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 32px rgba(79, 195, 247, 0.4);
}

.play-btn:active {
  transform: translateY(0);
}

.play-btn.active {
  background: linear-gradient(135deg, #1565C0, #1976D2);
  box-shadow: 0 4px 24px rgba(25, 118, 210, 0.3);
}

.btn-ring {
  position: absolute;
  inset: -2px;
  border-radius: 30px;
  border: 2px solid transparent;
  background: linear-gradient(135deg, rgba(255,255,255,0.3), transparent) border-box;
  -webkit-mask: linear-gradient(#fff 0 0) padding-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  opacity: 0;
  transition: opacity 0.3s;
}

.play-btn:hover:not(:disabled) .btn-ring {
  opacity: 1;
}

.btn-content {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 1px;
  position: relative;
  z-index: 1;
}

/* ===== Bottom Bar ===== */
.bottom-bar {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  font-size: 11px;
  color: var(--text-dim);
  opacity: 0.5;
}

/* ===== Side Panel ===== */
.side-panel {
  position: fixed;
  top: 0;
  right: 0;
  width: 340px;
  height: 100vh;
  background: rgba(17, 24, 39, 0.95);
  backdrop-filter: blur(20px);
  border-left: 1px solid var(--border);
  z-index: 1000;
  display: flex;
  flex-direction: column;
  animation: panelIn 0.25s ease;
}

@keyframes panelIn {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
}

.panel-header h2 {
  font-size: 15px;
  font-weight: 600;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* News */
.news-card {
  padding: 14px;
  background: rgba(255,255,255,0.03);
  border: 1px solid var(--border);
  border-radius: 10px;
}

.news-date {
  font-size: 11px;
  color: var(--text-dim);
  margin-bottom: 6px;
}

.news-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.news-desc {
  font-size: 12px;
  color: var(--text-dim);
}

/* Settings */
.setting-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
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
  height: 3px;
  background: rgba(255,255,255,0.1);
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
  box-shadow: 0 0 8px rgba(79, 195, 247, 0.4);
}

.setting-value {
  font-size: 13px;
  font-weight: 600;
  min-width: 50px;
  text-align: right;
}

.setting-dir {
  background: rgba(255,255,255,0.03);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 12px;
}

.dir-text {
  font-size: 12px;
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  color: var(--text-dim);
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
  background: rgba(255,255,255,0.08);
  border-radius: 10px;
  transition: background 0.2s;
  flex-shrink: 0;
}

.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  background: var(--text-dim);
  border-radius: 50%;
  transition: all 0.2s;
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
  animation: panelIn 0.25s ease;
}

.panel-leave-active {
  animation: panelIn 0.25s ease reverse;
}
</style>
