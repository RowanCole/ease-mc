<script>
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { listen } from '@tauri-apps/api/event'
  import { marked } from 'marked'
  import { Bot, Box, Gamepad2, Play, SendHorizontal, Square, X } from 'lucide-svelte'
  import './App.css'

  // 保留 AI 回复中的换行（流式逐字追加依赖换行渲染）
  marked.setOptions({ breaks: true })

  const gameInfo = {
    name: 'MC STARTER',
    title: 'Minecraft',
    subtitle: '进入属于你的方块世界',
  }

  const quickPrompts = ['新手应该先做什么？', '怎么找到钻石？', '下界要注意什么？']

  let status = 'downloading'
  let statusText = '游戏下载中...'
  let downloadPercent = 0
  let isExtracting = false
  let needDownload = false
  let showChat = false
  let draftMessage = ''
  let isReplying = false
  let streamStarted = false
  let chatMessages = [
    {
      id: 1,
      role: 'assistant',
      text: '你好！我是游戏助手。想聊聊生存、合成、探索，还是和朋友一起玩？',
    },
  ]
  let chatListEl
  // 记录当前流式输出的助手消息 id，用于逐字追加
  let streamingAssistantId = null

  $: isPlaying = status === 'playing'
  $: isDownloading = status === 'downloading'

  function scrollChatToEnd() {
    requestAnimationFrame(() => {
      chatListEl?.scrollTo({ top: chatListEl.scrollHeight, behavior: 'smooth' })
    })
  }

  function openChat() {
    showChat = true
    scrollChatToEnd()
  }

  // 下载游戏，启动检查与手动重试共用
  async function downloadGame() {
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
    })

    try {
      await invoke('download_game')
      needDownload = false
      status = 'ready'
      statusText = '游戏下载完成，可以开始冒险了'
      downloadPercent = 100
    } catch (error) {
      console.error('下载失败:', error)
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

  async function sendMessage(message = draftMessage) {
    const content = message.trim()
    if (!content || isReplying) return

    chatMessages = [...chatMessages, { id: Date.now(), role: 'user', text: content }]
    draftMessage = ''
    isReplying = true
    streamStarted = false
    scrollChatToEnd()

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
      scrollChatToEnd()
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
      scrollChatToEnd()
    })
    unlistenDone = await listen('chat-done', finish)

    try {
      await invoke('send_messages_to_mode', { message: content })
      finish()
    } catch (error) {
      console.error('AI 请求失败:', error)
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
    if (isPlaying) {
      try {
        await invoke('close_game')
        status = 'ready'
        statusText = '准备开始冒险'
      } catch (error) {
        console.error('Close failed:', error)
        statusText = '暂时无法结束游戏'
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
    }
  }

  function handleComposerKeyDown(event) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      sendMessage()
    }
  }

  onMount(() => {
    // 游戏进程被手动关闭时，恢复启动器为可开始状态
    const unlistenExited = listen('game-exited', () => {
      status = 'ready'
      statusText = '准备开始冒险'
    })
    ensureGameInstalled()

    return () => {
      unlistenExited.then((fn) => fn?.())
    }
  })
</script>

<div class="app-shell">
  <div class="scene" aria-hidden="true"></div>
  <div class="scene-overlay" aria-hidden="true"></div>
  <div class="scene-grid" aria-hidden="true"></div>

  <header class="topbar">
    <div class="brand-lockup">
      <div class="brand-mark" aria-hidden="true">
        <Box size={19} strokeWidth={2.3} />
      </div>
      <p>{gameInfo.name}</p>
    </div>
    <button class="assistant-trigger" type="button" title="打开游戏助手" on:click={openChat}>
      <span class="assistant-trigger-icon"><Bot size={19} strokeWidth={2.2} /></span>
      <span>游戏助手</span>
    </button>
  </header>

  <main class="launcher-main">
    <section class="hero-copy" aria-labelledby="launcher-title">
      <div class="eyebrow"><span></span> WELCOME BACK</div>
      <h1 id="launcher-title">{gameInfo.title}</h1>
      <p>{gameInfo.subtitle}</p>
    </section>

    <section class="launch-deck" aria-label="游戏启动">
      <div class="deck-intro">
        <div class="deck-symbol" aria-hidden="true">
          <Gamepad2 size={22} strokeWidth={2} />
        </div>
        <div>
          <p class="deck-state">{isDownloading ? '游戏下载中' : isPlaying ? '游戏正在运行' : needDownload ? '游戏未安装' : '已准备就绪'}</p>
          <p class="deck-hint">{isDownloading ? `下载进度 ${downloadPercent.toFixed(1)}%` : isPlaying ? '愿你的冒险一切顺利' : needDownload ? '游戏尚未安装' : '随时可以开始新的冒险'}</p>
        </div>
      </div>
      <h2>{isDownloading ? '正在准备游戏...' : isPlaying ? '愿你的冒险一切顺利' : needDownload ? '游戏尚未安装' : '准备好出发了吗？'}</h2>
      <p class="deck-copy">
        {isDownloading ? '游戏下载完成后，即可开始冒险。' : isPlaying ? '游戏正在运行。' : needDownload ? '点击下方按钮，重新下载并安装游戏。' : '点击下方按钮，即刻进入游戏。'}
      </p>

      <div class="launch-divider"></div>

      <button
        class="launch-button"
        class:running={isPlaying || isExtracting}
        class:downloading={isDownloading && !isExtracting}
        type="button"
        on:click={startGame}
        disabled={isDownloading}
        aria-busy={isDownloading}
      >
        {#if isDownloading}
          <span
            class="download-wave-svg"
            aria-hidden="true"
            style="animation: none; transform: translateY({100 - downloadPercent}%);"
          >
            <svg class="download-wave-layer download-wave-layer--back" viewBox="0 0 2000 100" preserveAspectRatio="none">
              <defs>
                <linearGradient id="download-wave-back" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="#a5e8f8" stop-opacity="0.24" />
                  <stop offset="100%" stop-color="#58c4e8" stop-opacity="0.26" />
                </linearGradient>
              </defs>
              <path d="M0 66 C83 59 167 59 250 66 C333 73 417 73 500 66 C583 59 667 59 750 66 C833 73 917 73 1000 66 C1083 59 1167 59 1250 66 C1333 73 1417 73 1500 66 C1583 59 1667 59 1750 66 C1833 73 1917 73 2000 66 V100 H0 Z" fill="url(#download-wave-back)" />
            </svg>
            <svg class="download-wave-layer download-wave-layer--mid" viewBox="0 0 2000 100" preserveAspectRatio="none">
              <defs>
                <linearGradient id="download-wave-mid" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="#91e3f6" stop-opacity="0.36" />
                  <stop offset="100%" stop-color="#58c4e8" stop-opacity="0.4" />
                </linearGradient>
              </defs>
              <path d="M0 64 C83 55 167 55 250 64 C333 73 417 73 500 64 C583 55 667 55 750 64 C833 73 917 73 1000 64 C1083 55 1167 55 1250 64 C1333 73 1417 73 1500 64 C1583 55 1667 55 1750 64 C1833 73 1917 73 2000 64 V100 H0 Z" fill="url(#download-wave-mid)" />
            </svg>
            <svg class="download-wave-layer download-wave-layer--front" viewBox="0 0 2000 100" preserveAspectRatio="none">
              <defs>
                <linearGradient id="download-wave-front" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="#c7f5ff" stop-opacity="0.7" />
                  <stop offset="58%" stop-color="#74d3ee" stop-opacity="0.74" />
                  <stop offset="100%" stop-color="#58c4e8" stop-opacity="0.78" />
                </linearGradient>
              </defs>
              <path d="M0 62 C83 52 167 52 250 62 C333 72 417 72 500 62 C583 52 667 52 750 62 C833 72 917 72 1000 62 C1083 52 1167 52 1250 62 C1333 72 1417 72 1500 62 C1583 52 1667 52 1750 62 C1833 72 1917 72 2000 62 V100 H0 Z" fill="url(#download-wave-front)" />
              <path d="M0 62 C83 52 167 52 250 62 C333 72 417 72 500 62 C583 52 667 52 750 62 C833 72 917 72 1000 62 C1083 52 1167 52 1250 62 C1333 72 1417 72 1500 62 C1583 52 1667 52 1750 62 C1833 72 1917 72 2000 62" class="download-wave-crest" />
            </svg>
          </span>
        {/if}
        <span class="launch-button-content">
          {#if isPlaying}
            <Square size={18} fill="currentColor" />
          {:else if !isDownloading}
            <Play size={20} fill="currentColor" />
          {/if}
          <span>{isExtracting ? '正在解压...' : isDownloading ? '游戏下载中...' : needDownload ? '下载游戏' : isPlaying ? '结束游戏' : '启动游戏'}</span>
        </span>
      </button>
      <p class="launch-status">
        <span class:running={isPlaying}></span>{statusText}
      </p>
    </section>
  </main>

  <footer class="footer-bar">© {new Date().getFullYear()} {gameInfo.name}</footer>

  {#if showChat}
    <button class="chat-backdrop" type="button" aria-label="关闭游戏助手" on:click={() => (showChat = false)}></button>
  {/if}

  {#if showChat}
    <aside class="chat-panel" aria-label="游戏助手">
      <header class="chat-header">
        <div class="chat-title">
          <span class="chat-title-icon"><Bot size={19} strokeWidth={2.2} /></span>
          <div>
            <p>游戏助手</p>
            <span>随时问我游戏里的事</span>
          </div>
        </div>
        <button class="chat-icon-button" type="button" title="关闭游戏助手" aria-label="关闭游戏助手" on:click={() => (showChat = false)}>
          <X size={18} />
        </button>
      </header>

      <div bind:this={chatListEl} class="chat-messages" aria-live="polite">
        {#each chatMessages as message (message.id)}
          <div class="chat-message {message.role}">
            {#if message.role === 'assistant'}
              <span class="assistant-avatar"><Bot size={15} strokeWidth={2.2} /></span>
              <div class="chat-bubble markdown">{@html marked.parse(message.text)}</div>
            {:else}
              <p>{message.text}</p>
            {/if}
          </div>
        {/each}
        {#if isReplying && !streamStarted}
          <div class="chat-message assistant thinking" aria-label="游戏助手正在输入">
            <span class="assistant-avatar"><Bot size={15} strokeWidth={2.2} /></span>
            <p><i></i><i></i><i></i></p>
          </div>
        {/if}
      </div>

      {#if chatMessages.length === 1}
        <div class="quick-prompts" aria-label="推荐问题">
          {#each quickPrompts as prompt (prompt)}
            <button type="button" on:click={() => sendMessage(prompt)}>{prompt}</button>
          {/each}
        </div>
      {/if}

      <form class="chat-composer" on:submit|preventDefault={() => sendMessage()}>
        <textarea
          rows="1"
          maxlength="300"
          placeholder="问问游戏里的事..."
          bind:value={draftMessage}
          on:keydown={handleComposerKeyDown}
        ></textarea>
        <button type="submit" title="发送消息" aria-label="发送消息" disabled={!draftMessage.trim() || isReplying}>
          <SendHorizontal size={18} />
        </button>
      </form>
    </aside>
  {/if}
</div>
