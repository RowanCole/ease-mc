<script>
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { listen } from '@tauri-apps/api/event'
  import './App.css'

  import { gameInfo, isTauri } from './constants.js'
  import Scene from './components/Scene.svelte'
  import TopBar from './components/TopBar.svelte'
  import HeroSection from './components/HeroSection.svelte'
  import LaunchCard from './components/LaunchCard.svelte'
  import ChatPanel from './components/ChatPanel.svelte'
  import ToastStack from './components/ToastStack.svelte'

  let status = 'downloading'
  let statusText = '游戏下载中...'
  let downloadPercent = 0
  let isExtracting = false
  let needDownload = false
  let showChat = false
  let isReplying = false
  let streamStarted = false
  let chatMessages = [
    {
      id: 1,
      role: 'assistant',
      text: '你好！我是游戏助手。想聊聊生存、合成、探索，还是和朋友一起玩？',
    },
  ]
  // 记录当前流式输出的助手消息 id，用于逐字追加
  let streamingAssistantId = null
  // 弹窗消息通知（下载/启动/AI 等异常时提醒用户）
  let toasts = []
  let toastId = 0
  // 同屏最多展示的弹窗数量，超出时顶掉最早的
  const MAX_TOASTS = 3

  function errorText(error) {
    return typeof error === 'string' ? error : error?.message ?? String(error)
  }

  function notify(message, type = 'error') {
    // 相同消息已在展示中则忽略，避免重复点击导致弹窗堆积
    if (toasts.some((t) => t.message === message && t.type === type)) return

    const id = ++toastId
    toasts = [...toasts, { id, message, type }]
    if (toasts.length > MAX_TOASTS) {
      toasts = toasts.slice(toasts.length - MAX_TOASTS)
    }
    setTimeout(() => dismissToast(id), 5000)
  }

  function dismissToast(id) {
    toasts = toasts.filter((t) => t.id !== id)
  }

  $: isPlaying = status === 'playing'
  $: isDownloading = status === 'downloading'

  function openChat() {
    showChat = true
  }

  // 下载游戏，启动检查与手动重试共用
  async function downloadGame() {
    if (!isTauri) {
      notify('浏览器预览模式，无法执行下载', 'info')
      return
    }
    // 防重入：下载/安装过程中忽略重复点击
    if (isDownloading || isExtracting) {
      notify('游戏正在下载/安装中，请稍候', 'info')
      return
    }
    status = 'downloading'
    statusText = '游戏下载中...'
    downloadPercent = 0

    const unlistenProgress = await listen('download-progress', (event) => {
      downloadPercent = event.payload.percent
    })
    // 监听运行环境安装事件（JRE 下载解压阶段），切换按钮文案与颜色
    const unlistenExtract = await listen('extract-start', () => {
      isExtracting = true
      statusText = '正在安装运行环境...'
      notify('正在安装运行环境，请稍候...', 'info')
    })

    try {
      await invoke('download_game')
      needDownload = false
      status = 'ready'
      statusText = '游戏下载完成，可以开始冒险了'
      downloadPercent = 100
      notify('游戏下载完成，可以开始冒险了', 'success')
    } catch (error) {
      console.error('下载失败:', error)
      notify(`游戏下载失败：${errorText(error)}`, 'error')
      needDownload = true
      status = 'ready'
      statusText = '游戏下载失败，请检查网络后重试'
    } finally {
      isExtracting = false
      unlistenProgress()
      unlistenExtract()
    }
  }

  // 应用启动时检查游戏是否已安装，未安装则自动下载
  async function ensureGameInstalled() {
    if (!isTauri) {
      status = 'ready'
      statusText = '浏览器预览模式（仅用于界面调试）'
      return
    }
    let installed = ''
    try {
      installed = await invoke('get_config', { key: 'gameIsInstalled' })
    } catch (error) {
      console.error('读取配置失败:', error)
    }

    if (installed === 'false') {
      needDownload = true
      await downloadGame()
    } else {
      status = 'ready'
      statusText = '游戏下载完成，可以开始冒险了'
    }
  }

  // 发送 AI 消息：输入框与消息列表由 ChatPanel 管理，这里负责流式接收与状态更新
  async function sendMessage(message) {
    if (!isTauri) {
      notify('浏览器预览模式，无法与 AI 对话', 'info')
      return
    }
    const content = message.trim()
    if (!content || isReplying) return

    chatMessages = [...chatMessages, { id: Date.now(), role: 'user', text: content }]
    isReplying = true
    streamStarted = false

    let finished = false
    let unlistenChunk
    let unlistenDone

    const finish = () => {
      if (finished) return
      finished = true
      streamingAssistantId = null
      unlistenChunk?.()
      unlistenDone?.()
      isReplying = false
    }

    // 流式监听：首块到达时创建助手消息，后续逐字追加
    unlistenChunk = await listen('chat-chunk', (event) => {
      const chunk = event.payload
      if (streamingAssistantId == null) {
        const newId = Date.now()
        streamingAssistantId = newId
        streamStarted = true
        chatMessages = [...chatMessages, { id: newId, role: 'assistant', text: chunk }]
      } else {
        chatMessages = chatMessages.map((m) =>
          m.id === streamingAssistantId ? { ...m, text: m.text + chunk } : m,
        )
      }
    })
    unlistenDone = await listen('chat-done', finish)

    try {
      await invoke('send_messages_to_mode', { message: content })
      finish()
    } catch (error) {
      console.error('AI 请求失败:', error)
      notify(`AI 请求失败：${errorText(error)}`, 'error')
      if (streamingAssistantId == null) {
        chatMessages = [
          ...chatMessages,
          { id: Date.now(), role: 'assistant', text: '抱歉，AI 暂时无法回答，请稍后重试。' },
        ]
      } else {
        chatMessages = chatMessages.map((m) =>
          m.id === streamingAssistantId ? { ...m, text: m.text + '（请求中断，请重试）' } : m,
        )
      }
      finish()
    }
  }

  async function startGame() {
    if (!isTauri) {
      notify('浏览器预览模式，无法启动游戏', 'info')
      return
    }
    if (isPlaying) {
      try {
        await invoke('close_game')
        status = 'ready'
        statusText = '准备开始冒险'
      } catch (error) {
        console.error('Close failed:', error)
        statusText = '暂时无法结束游戏'
        notify(`无法结束游戏：${errorText(error)}`, 'error')
      }
      return
    }

    if (needDownload) {
      await downloadGame()
      return
    }

    statusText = '正在启动游戏'

    try {
      await invoke('launch_game')
      status = 'playing'
      statusText = '游戏正在运行'
    } catch (error) {
      console.error('Launch failed:', error)
      statusText = '启动失败，请稍后重试'
      notify(`游戏启动失败：${errorText(error)}`, 'error')
    }
  }

  onMount(() => {
    let unlistenExited
    if (isTauri) {
      // 游戏进程被手动关闭时，恢复启动器为可开始状态
      unlistenExited = listen('game-exited', () => {
        status = 'ready'
        statusText = '准备开始冒险'
      })
    }
    ensureGameInstalled()

    return () => {
      unlistenExited?.then((fn) => fn?.())
    }
  })
</script>

<div class="app-shell">
  <Scene />

  <TopBar gameName={gameInfo.name} on:openAssistant={openChat} />

  <main class="launcher-main">
    <HeroSection title={gameInfo.title} subtitle={gameInfo.subtitle} />
    <LaunchCard
      {isDownloading}
      {isPlaying}
      {needDownload}
      {downloadPercent}
      {isExtracting}
      {statusText}
      on:action={startGame}
    />
  </main>

  <footer class="footer-bar">© {new Date().getFullYear()} {gameInfo.name}</footer>

  {#if showChat}
    <ChatPanel
      {chatMessages}
      {isReplying}
      {streamStarted}
      on:close={() => (showChat = false)}
      on:send={(event) => sendMessage(event.detail)}
    />
  {/if}

  <ToastStack {toasts} on:dismiss={(event) => dismissToast(event.detail)} />
</div>
