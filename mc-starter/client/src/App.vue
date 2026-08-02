<script setup>
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Box, Gamepad2, Play, Square } from 'lucide-vue-next'

const status = ref('ready')
const statusText = ref('准备开始冒险')

const gameInfo = {
  name: 'MC STARTER',
  title: 'Minecraft',
  subtitle: '进入属于你的方块世界',
}

const isPlaying = computed(() => status.value === 'playing')

async function startGame() {
  if (isPlaying.value) {
    try {
      await invoke('close_game')
      status.value = 'ready'
      statusText.value = '准备开始冒险'
    } catch (error) {
      console.error('Close failed:', error)
      statusText.value = '暂时无法结束游戏'
    }
    return
  }

  statusText.value = '正在启动游戏'

  try {
    await invoke('launch_game')
    status.value = 'playing'
    statusText.value = '游戏正在运行'
  } catch (error) {
    console.error('Launch failed:', error)
    statusText.value = '启动失败，请稍后重试'
  }
}
</script>

<template>
  <div class="app-shell">
    <div class="scene" aria-hidden="true"></div>
    <div class="scene-overlay" aria-hidden="true"></div>
    <div class="scene-grid" aria-hidden="true"></div>

    <header class="topbar">
      <div class="brand-lockup">
        <div class="brand-mark" aria-hidden="true">
          <Box :size="19" :stroke-width="2.3" />
        </div>
        <p>{{ gameInfo.name }}</p>
      </div>
    </header>

    <main class="launcher-main">
      <section class="hero-copy" aria-labelledby="launcher-title">
        <div class="eyebrow"><span></span> WELCOME BACK</div>
        <h1 id="launcher-title">{{ gameInfo.title }}</h1>
        <p>{{ gameInfo.subtitle }}</p>
      </section>

      <section class="launch-deck" aria-label="游戏启动">
        <div class="deck-intro">
          <div class="deck-symbol" aria-hidden="true">
            <Gamepad2 :size="22" :stroke-width="2" />
          </div>
          <div>
            <p class="deck-state">{{ isPlaying ? '游戏正在运行' : '已准备就绪' }}</p>
            <p class="deck-hint">{{ isPlaying ? '愿你的冒险一切顺利' : '随时可以开始新的冒险' }}</p>
          </div>
        </div>
        <h2>{{ isPlaying ? '愿你的冒险一切顺利' : '准备好出发了吗？' }}</h2>
        <p class="deck-copy">
          {{ isPlaying ? '游戏正在运行。' : '点击下方按钮，即刻进入游戏。' }}
        </p>

        <div class="launch-divider"></div>

        <button class="launch-button" :class="{ running: isPlaying }" type="button" @click="startGame">
          <Square v-if="isPlaying" :size="18" fill="currentColor" />
          <Play v-else :size="20" fill="currentColor" />
          <span>{{ isPlaying ? '结束游戏' : '开始游戏' }}</span>
        </button>
        <p class="launch-status">
          <span :class="{ running: isPlaying }"></span>{{ statusText }}
        </p>
      </section>
    </main>

    <footer class="footer-bar">© {{ new Date().getFullYear() }} {{ gameInfo.name }}</footer>
  </div>
</template>

<style>
*,
*::before,
*::after {
  box-sizing: border-box;
}

:root {
  color: #f4f6ef;
  background: #101611;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans SC", sans-serif;
  font-synthesis: none;
}

html,
body,
#app {
  min-width: 100%;
  min-height: 100%;
  margin: 0;
}

html,
body {
  overflow: hidden;
}

button {
  font: inherit;
  -webkit-tap-highlight-color: transparent;
}
</style>

<style scoped>
.app-shell {
  --ink: #f5f7f0;
  --muted: #c3cec0;
  --quiet: #8e9c8d;
  --line: rgba(234, 242, 227, 0.17);
  --lime: #b6ef77;
  --lime-bright: #d9ffab;
  --amber: #ffc276;
  position: relative;
  isolation: isolate;
  display: flex;
  min-height: 100vh;
  flex-direction: column;
  overflow: hidden;
}

.scene,
.scene-overlay,
.scene-grid {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.scene {
  z-index: -3;
  background: url('/src/assets/background.jpg') center / cover no-repeat;
  transform: scale(1.02);
}

.scene-overlay {
  z-index: -2;
  background:
    linear-gradient(90deg, rgba(11, 18, 12, 0.91) 0%, rgba(12, 20, 13, 0.72) 38%, rgba(12, 18, 12, 0.28) 72%, rgba(7, 12, 8, 0.5) 100%),
    linear-gradient(0deg, rgba(6, 10, 7, 0.86) 0%, transparent 46%, rgba(6, 10, 7, 0.15) 100%);
}

.scene-grid {
  z-index: -1;
  opacity: 0.16;
  background-image: linear-gradient(rgba(255, 255, 255, 0.04) 1px, transparent 1px), linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px);
  background-size: 36px 36px;
  mask-image: linear-gradient(to bottom, transparent, black 14%, black 88%, transparent);
}

.topbar {
  display: flex;
  min-height: 82px;
  align-items: center;
  padding: 0 clamp(24px, 4vw, 64px);
  background: rgba(9, 15, 10, 0.34);
  border-bottom: 1px solid rgba(234, 242, 227, 0.1);
  -webkit-app-region: drag;
}

.brand-lockup {
  display: flex;
  align-items: center;
  gap: 11px;
}

.brand-mark {
  display: grid;
  width: 35px;
  height: 35px;
  place-items: center;
  color: #14200f;
  background: var(--lime);
  border: 1px solid rgba(244, 255, 227, 0.62);
  border-radius: 6px;
  box-shadow: 0 7px 18px rgba(100, 153, 50, 0.25);
}

.brand-lockup p {
  margin: 0;
  color: var(--ink);
  font-size: 13px;
  font-weight: 850;
  letter-spacing: 0.08em;
}

.launcher-main {
  display: grid;
  width: min(100% - 48px, 1120px);
  grid-template-columns: minmax(0, 1.25fr) minmax(310px, 0.75fr);
  gap: clamp(48px, 9vw, 150px);
  margin: auto;
  padding: 44px 0;
}

.hero-copy {
  align-self: center;
  max-width: 640px;
}

.eyebrow {
  display: flex;
  align-items: center;
  gap: 9px;
  color: var(--lime-bright);
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.1em;
}

.eyebrow span {
  width: 28px;
  height: 1px;
  background: currentColor;
}

.hero-copy h1 {
  margin: 17px 0 13px;
  color: var(--ink);
  font-size: clamp(50px, 7vw, 88px);
  font-weight: 850;
  line-height: 0.94;
  letter-spacing: 0;
}

.hero-copy > p {
  margin: 0;
  color: var(--muted);
  font-size: 16px;
  line-height: 1.6;
}

.launch-deck {
  align-self: center;
  padding: 32px 34px 25px;
  color: var(--ink);
  text-align: left;
  background: rgba(16, 25, 17, 0.83);
  border: 1px solid var(--line);
  border-radius: 8px;
  box-shadow: 0 24px 56px rgba(0, 0, 0, 0.28), inset 0 1px 0 rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(18px) saturate(105%);
}

.deck-intro {
  display: flex;
  align-items: center;
  gap: 13px;
  margin-bottom: 27px;
}

.deck-symbol {
  display: grid;
  width: 46px;
  height: 46px;
  flex: 0 0 auto;
  place-items: center;
  color: #1a2713;
  background: var(--lime);
  border: 1px solid rgba(248, 255, 238, 0.58);
  border-radius: 6px;
  box-shadow: 0 8px 19px rgba(122, 181, 63, 0.22);
}

.deck-state {
  margin: 0;
  color: var(--lime-bright);
  font-size: 12px;
  font-weight: 780;
}

.deck-hint {
  margin: 4px 0 0;
  color: var(--quiet);
  font-size: 12px;
  line-height: 1.35;
}

.launch-deck h2 {
  margin: 0 0 9px;
  color: var(--ink);
  font-size: 25px;
  font-weight: 760;
  line-height: 1.2;
}

.deck-copy {
  min-height: 23px;
  margin: 0;
  color: var(--muted);
  font-size: 14px;
  line-height: 1.6;
}

.launch-divider {
  height: 1px;
  margin: 29px 0 20px;
  background: rgba(234, 242, 227, 0.12);
}

.launch-button {
  display: inline-flex;
  width: 100%;
  min-height: 58px;
  align-items: center;
  justify-content: center;
  gap: 9px;
  padding: 0 18px;
  color: #13200e;
  background: var(--lime);
  border: 1px solid rgba(248, 255, 238, 0.65);
  border-radius: 5px;
  box-shadow: 0 12px 26px rgba(109, 169, 52, 0.25);
  cursor: pointer;
  font-size: 16px;
  font-weight: 850;
  transition: background-color 160ms ease, box-shadow 160ms ease, transform 160ms ease;
}

.launch-button:hover {
  background: var(--lime-bright);
  box-shadow: 0 15px 30px rgba(109, 169, 52, 0.35);
  transform: translateY(-1px);
}

.launch-button:active {
  transform: translateY(0);
}

.launch-button.running {
  color: #382006;
  background: var(--amber);
  box-shadow: 0 12px 26px rgba(223, 143, 52, 0.22);
}

.launch-button:focus-visible {
  outline: 2px solid var(--lime-bright);
  outline-offset: 3px;
}

.launch-status {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  max-width: 100%;
  margin: 15px 0 0;
  color: var(--quiet);
  font-size: 11px;
  font-weight: 650;
}

.launch-status > span {
  width: 6px;
  height: 6px;
  flex: 0 0 auto;
  background: var(--lime);
  border-radius: 50%;
  box-shadow: 0 0 0 3px rgba(182, 239, 119, 0.13);
}

.launch-status > span.running {
  background: var(--amber);
  box-shadow: 0 0 0 3px rgba(255, 194, 118, 0.13);
}

.footer-bar {
  display: flex;
  min-height: 47px;
  align-items: center;
  justify-content: center;
  color: rgba(235, 243, 230, 0.55);
  background: rgba(6, 10, 7, 0.34);
  border-top: 1px solid rgba(234, 242, 227, 0.1);
  font-size: 10px;
  font-weight: 650;
  letter-spacing: 0.07em;
}

@media (max-width: 800px) {
  .launcher-main {
    width: min(100% - 40px, 560px);
    grid-template-columns: 1fr;
    gap: 38px;
    padding-block: 39px 31px;
  }

  .hero-copy {
    align-self: end;
  }
}

@media (max-width: 530px) {
  .topbar {
    min-height: 66px;
    padding-inline: 16px;
  }

  .launcher-main {
    width: min(100% - 32px, 560px);
    padding-top: 31px;
  }

  .hero-copy h1 {
    font-size: 51px;
  }

  .hero-copy > p {
    font-size: 14px;
  }

  .launch-deck {
    padding: 28px 22px 23px;
  }
}

@media (max-height: 650px) and (min-width: 801px) {
  .launcher-main {
    padding-block: 25px;
  }

  .launch-deck {
    padding-top: 23px;
  }

  .deck-symbol {
    width: 42px;
    height: 42px;
  }

  .deck-intro {
    margin-bottom: 19px;
  }

  .launch-divider {
    margin-block: 19px 16px;
  }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    scroll-behavior: auto !important;
    transition-duration: 0.01ms !important;
  }
}
</style>
