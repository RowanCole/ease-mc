<script setup>
import { computed, nextTick, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Bot, Box, Gamepad2, Play, SendHorizontal, Square, X } from 'lucide-vue-next'

const status = ref('ready')
const statusText = ref('准备开始冒险')
const showChat = ref(false)
const draftMessage = ref('')
const isReplying = ref(false)
const chatList = ref(null)
const chatMessages = ref([
  {
    id: 1,
    role: 'assistant',
    text: '你好！我是游戏助手。想聊聊生存、合成、探索，还是和朋友一起玩？',
  },
])

const gameInfo = {
  name: 'MC STARTER',
  title: 'Minecraft',
  subtitle: '进入属于你的方块世界',
}

const isPlaying = computed(() => status.value === 'playing')

const quickPrompts = ['新手应该先做什么？', '怎么找到钻石？', '下界要注意什么？']

function getAssistantReply(message) {
  const question = message.toLowerCase()

  if (question.includes('新手') || question.includes('开始') || question.includes('生存') || question.includes('怎么玩')) {
    return '先收集木头，做出工作台和基础工具。天黑前准备好食物与临时住处，第一晚就会轻松很多。'
  }

  if (question.includes('钻石')) {
    return '钻石通常出现在较深的地下。带上足够的火把、食物和铁镐，沿着洞穴或矿道慢慢探索会更安全。'
  }

  if (question.includes('下界')) {
    return '进入下界前，先准备食物、金质装备和备用方块。那里地形危险，记得标记传送门的位置。'
  }

  if (question.includes('联机') || question.includes('朋友') || question.includes('一起')) {
    return '和朋友一起玩时，可以先分工收集资源、建造基地和探索地图。把重要物资放进公共箱子会方便很多。'
  }

  if (question.includes('合成') || question.includes('工作台')) {
    return '四个木板可以合成工作台。工作台能制作大部分常用物品，是开始冒险后的第一件重要工具。'
  }

  return '我可以帮你聊生存技巧、合成思路、探索路线和联机玩法。换个问法再告诉我你的困惑吧。'
}

async function scrollChatToEnd() {
  await nextTick()
  chatList.value?.scrollTo({ top: chatList.value.scrollHeight, behavior: 'smooth' })
}

function openChat() {
  showChat.value = true
  scrollChatToEnd()
}

async function sendMessage(message = draftMessage.value) {
  const content = message.trim()
  if (!content || isReplying.value) return

  chatMessages.value.push({ id: Date.now(), role: 'user', text: content })
  draftMessage.value = ''
  isReplying.value = true
  await scrollChatToEnd()

  await new Promise((resolve) => window.setTimeout(resolve, 420))
  chatMessages.value.push({ id: Date.now() + 1, role: 'assistant', text: getAssistantReply(content) })
  isReplying.value = false
  await scrollChatToEnd()
}

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
      <button class="assistant-trigger" type="button" title="打开游戏助手" @click="openChat">
        <span class="assistant-trigger-icon"><Bot :size="19" :stroke-width="2.2" /></span>
        <span>游戏助手</span>
      </button>
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

    <Transition name="chat-backdrop">
      <button v-if="showChat" class="chat-backdrop" type="button" aria-label="关闭游戏助手" @click="showChat = false"></button>
    </Transition>
    <Transition name="chat-panel">
      <aside v-if="showChat" class="chat-panel" aria-label="游戏助手">
        <header class="chat-header">
          <div class="chat-title">
            <span class="chat-title-icon"><Bot :size="19" :stroke-width="2.2" /></span>
            <div>
              <p>游戏助手</p>
              <span>随时问我游戏里的事</span>
            </div>
          </div>
          <button class="chat-icon-button" type="button" title="关闭游戏助手" aria-label="关闭游戏助手" @click="showChat = false">
            <X :size="18" />
          </button>
        </header>

        <div ref="chatList" class="chat-messages" aria-live="polite">
          <div v-for="message in chatMessages" :key="message.id" class="chat-message" :class="message.role">
            <span v-if="message.role === 'assistant'" class="assistant-avatar"><Bot :size="15" :stroke-width="2.2" /></span>
            <p>{{ message.text }}</p>
          </div>
          <div v-if="isReplying" class="chat-message assistant thinking" aria-label="游戏助手正在输入">
            <span class="assistant-avatar"><Bot :size="15" :stroke-width="2.2" /></span>
            <p><i></i><i></i><i></i></p>
          </div>
        </div>

        <div v-if="chatMessages.length === 1" class="quick-prompts" aria-label="推荐问题">
          <button v-for="prompt in quickPrompts" :key="prompt" type="button" @click="sendMessage(prompt)">
            {{ prompt }}
          </button>
        </div>

        <form class="chat-composer" @submit.prevent="sendMessage()">
          <textarea v-model="draftMessage" rows="1" maxlength="300" placeholder="问问游戏里的事..." @keydown.enter.exact.prevent="sendMessage()"></textarea>
          <button type="submit" title="发送消息" aria-label="发送消息" :disabled="!draftMessage.trim() || isReplying">
            <SendHorizontal :size="18" />
          </button>
        </form>
      </aside>
    </Transition>
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
  justify-content: space-between;
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

.assistant-trigger {
  display: inline-flex;
  min-height: 36px;
  align-items: center;
  gap: 8px;
  padding: 3px 12px 3px 4px;
  color: var(--ink);
  background: rgba(234, 242, 227, 0.09);
  border: 1px solid rgba(234, 242, 227, 0.16);
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 720;
  -webkit-app-region: no-drag;
  transition: background-color 160ms ease, border-color 160ms ease, transform 160ms ease;
}

.assistant-trigger-icon {
  display: grid;
  width: 28px;
  height: 28px;
  place-items: center;
  color: #172212;
  background: var(--lime);
  border-radius: 4px;
}

.assistant-trigger:hover {
  background: rgba(182, 239, 119, 0.14);
  border-color: rgba(182, 239, 119, 0.28);
  transform: translateY(-1px);
}

.assistant-trigger:focus-visible,
.chat-icon-button:focus-visible,
.chat-composer textarea:focus-visible,
.chat-composer button:focus-visible,
.quick-prompts button:focus-visible {
  outline: 2px solid var(--lime-bright);
  outline-offset: 3px;
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

.chat-backdrop {
  position: fixed;
  z-index: 10;
  inset: 0;
  width: 100%;
  height: 100%;
  padding: 0;
  background: rgba(5, 10, 6, 0.28);
  border: 0;
  cursor: default;
}

.chat-panel {
  position: fixed;
  z-index: 11;
  top: 96px;
  right: clamp(16px, 3vw, 38px);
  display: flex;
  width: min(390px, calc(100vw - 32px));
  height: min(590px, calc(100dvh - 120px));
  min-height: 420px;
  flex-direction: column;
  overflow: hidden;
  color: var(--ink);
  background: rgba(15, 23, 16, 0.97);
  border: 1px solid rgba(234, 242, 227, 0.18);
  border-radius: 8px;
  box-shadow: 0 28px 70px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px) saturate(110%);
}

.chat-header {
  display: flex;
  min-height: 76px;
  align-items: center;
  justify-content: space-between;
  padding: 16px 18px 15px 20px;
  border-bottom: 1px solid rgba(234, 242, 227, 0.1);
}

.chat-title {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 10px;
}

.chat-title-icon,
.assistant-avatar {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  color: #1a2713;
  background: var(--lime);
}

.chat-title-icon {
  width: 36px;
  height: 36px;
  border-radius: 5px;
}

.chat-title > div > p,
.chat-title > div > span {
  display: block;
  margin: 0;
}

.chat-title > div > p {
  color: var(--ink);
  font-size: 14px;
  font-weight: 800;
}

.chat-title > div > span {
  margin-top: 3px;
  color: var(--quiet);
  font-size: 11px;
}

.chat-icon-button {
  display: grid;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  padding: 0;
  place-items: center;
  color: var(--muted);
  background: transparent;
  border: 1px solid transparent;
  border-radius: 5px;
  cursor: pointer;
  transition: background-color 160ms ease, color 160ms ease;
}

.chat-icon-button:hover {
  color: var(--ink);
  background: rgba(234, 242, 227, 0.1);
}

.chat-messages {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 13px;
  overflow-y: auto;
  padding: 20px;
  scrollbar-color: rgba(182, 239, 119, 0.35) transparent;
}

.chat-message {
  display: flex;
  max-width: 88%;
  align-items: center;
  gap: 8px;
}

.chat-message p {
  margin: 0;
  padding: 10px 12px;
  color: #dce5d8;
  background: rgba(234, 242, 227, 0.09);
  border: 1px solid rgba(234, 242, 227, 0.08);
  border-radius: 6px;
  font-size: 13px;
  line-height: 1.55;
}

.chat-message.user {
  align-self: flex-end;
}

.chat-message.user p {
  color: #14200f;
  background: var(--lime);
  border-color: rgba(248, 255, 238, 0.56);
}

.assistant-avatar {
  width: 27px;
  height: 27px;
  border-radius: 5px;
}

.thinking p {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 57px;
  min-height: 37px;
}

.thinking i {
  width: 5px;
  height: 5px;
  background: var(--lime);
  border-radius: 50%;
  animation: chat-bounce 1s infinite ease-in-out;
}

.thinking i:nth-child(2) {
  animation-delay: 120ms;
}

.thinking i:nth-child(3) {
  animation-delay: 240ms;
}

.quick-prompts {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
  padding: 0 20px 14px;
}

.quick-prompts button {
  min-height: 29px;
  padding: 5px 9px;
  color: var(--muted);
  background: rgba(234, 242, 227, 0.06);
  border: 1px solid rgba(234, 242, 227, 0.13);
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  transition: background-color 160ms ease, border-color 160ms ease, color 160ms ease;
}

.quick-prompts button:hover {
  color: var(--ink);
  background: rgba(182, 239, 119, 0.11);
  border-color: rgba(182, 239, 119, 0.3);
}

.chat-composer {
  display: flex;
  align-items: flex-end;
  gap: 9px;
  padding: 13px 14px 14px 16px;
  background: rgba(5, 10, 6, 0.22);
  border-top: 1px solid rgba(234, 242, 227, 0.1);
}

.chat-composer textarea {
  width: 100%;
  min-height: 38px;
  max-height: 76px;
  padding: 9px 0;
  resize: none;
  color: var(--ink);
  background: transparent;
  border: 0;
  outline: 0;
  font-size: 13px;
  line-height: 1.5;
}

.chat-composer textarea::placeholder {
  color: var(--quiet);
}

.chat-composer button {
  display: grid;
  width: 38px;
  height: 38px;
  flex: 0 0 auto;
  padding: 0;
  place-items: center;
  color: #13200e;
  background: var(--lime);
  border: 1px solid rgba(248, 255, 238, 0.58);
  border-radius: 5px;
  cursor: pointer;
  transition: background-color 160ms ease, opacity 160ms ease, transform 160ms ease;
}

.chat-composer button:hover:not(:disabled) {
  background: var(--lime-bright);
  transform: translateY(-1px);
}

.chat-composer button:disabled {
  cursor: not-allowed;
  opacity: 0.43;
}

@keyframes chat-bounce {
  0%, 100% { transform: translateY(0); opacity: 0.55; }
  50% { transform: translateY(-3px); opacity: 1; }
}

.chat-panel-enter-active,
.chat-panel-leave-active {
  transition: opacity 180ms ease, transform 200ms ease;
}

.chat-panel-enter-from,
.chat-panel-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.98);
}

.chat-backdrop-enter-active,
.chat-backdrop-leave-active {
  transition: opacity 180ms ease;
}

.chat-backdrop-enter-from,
.chat-backdrop-leave-to {
  opacity: 0;
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

  .assistant-trigger {
    width: 38px;
    padding: 4px;
    justify-content: center;
  }

  .assistant-trigger span {
    display: none;
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

  .chat-panel {
    top: auto;
    right: 16px;
    bottom: 16px;
    height: min(620px, calc(100dvh - 32px));
    min-height: 0;
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
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
  }
}
</style>
