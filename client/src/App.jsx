import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Bot, Box, Gamepad2, Play, SendHorizontal, Square, X } from 'lucide-react'
import './App.css'

const gameInfo = {
  name: 'MC STARTER',
  title: 'Minecraft',
  subtitle: '进入属于你的方块世界',
}

const quickPrompts = ['新手应该先做什么？', '怎么找到钻石？', '下界要注意什么？']

// 防止 React StrictMode 开发模式下重复触发下载
let installCheckStarted = false

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

function DownloadWaveSvg({ percent }) {
  return (
    <span
      className="download-wave-svg"
      aria-hidden="true"
      style={{
        animation: 'none',
        transform: `translateY(${100 - percent}%)`,
      }}
    >
      <svg className="download-wave-layer download-wave-layer--back" viewBox="0 0 2000 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="download-wave-back" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#a5e8f8" stopOpacity="0.24" />
            <stop offset="100%" stopColor="#58c4e8" stopOpacity="0.26" />
          </linearGradient>
        </defs>
        <path d="M0 66 C83 59 167 59 250 66 C333 73 417 73 500 66 C583 59 667 59 750 66 C833 73 917 73 1000 66 C1083 59 1167 59 1250 66 C1333 73 1417 73 1500 66 C1583 59 1667 59 1750 66 C1833 73 1917 73 2000 66 V100 H0 Z" fill="url(#download-wave-back)" />
      </svg>
      <svg className="download-wave-layer download-wave-layer--mid" viewBox="0 0 2000 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="download-wave-mid" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#91e3f6" stopOpacity="0.36" />
            <stop offset="100%" stopColor="#58c4e8" stopOpacity="0.4" />
          </linearGradient>
        </defs>
        <path d="M0 64 C83 55 167 55 250 64 C333 73 417 73 500 64 C583 55 667 55 750 64 C833 73 917 73 1000 64 C1083 55 1167 55 1250 64 C1333 73 1417 73 1500 64 C1583 55 1667 55 1750 64 C1833 73 1917 73 2000 64 V100 H0 Z" fill="url(#download-wave-mid)" />
      </svg>
      <svg className="download-wave-layer download-wave-layer--front" viewBox="0 0 2000 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="download-wave-front" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#c7f5ff" stopOpacity="0.7" />
            <stop offset="58%" stopColor="#74d3ee" stopOpacity="0.74" />
            <stop offset="100%" stopColor="#58c4e8" stopOpacity="0.78" />
          </linearGradient>
        </defs>
        <path d="M0 62 C83 52 167 52 250 62 C333 72 417 72 500 62 C583 52 667 52 750 62 C833 72 917 72 1000 62 C1083 52 1167 52 1250 62 C1333 72 1417 72 1500 62 C1583 52 1667 52 1750 62 C1833 72 1917 72 2000 62 V100 H0 Z" fill="url(#download-wave-front)" />
        <path d="M0 62 C83 52 167 52 250 62 C333 72 417 72 500 62 C583 52 667 52 750 62 C833 72 917 72 1000 62 C1083 52 1167 52 1250 62 C1333 72 1417 72 1500 62 C1583 52 1667 52 1750 62 C1833 72 1917 72 2000 62" className="download-wave-crest" />
      </svg>
    </span>
  )
}

export default function App() {
  const [status, setStatus] = useState('downloading')
  const [statusText, setStatusText] = useState('游戏下载中...')
  const [downloadPercent, setDownloadPercent] = useState(0)
  const [showChat, setShowChat] = useState(false)
  const [draftMessage, setDraftMessage] = useState('')
  const [isReplying, setIsReplying] = useState(false)
  const [chatMessages, setChatMessages] = useState([
    {
      id: 1,
      role: 'assistant',
      text: '你好！我是游戏助手。想聊聊生存、合成、探索，还是和朋友一起玩？',
    },
  ])
  const chatListRef = useRef(null)

  const isPlaying = status === 'playing'
  const isDownloading = status === 'downloading'

  // 应用启动时检查游戏是否已安装，未安装则自动下载
  useEffect(() => {
    async function ensureGameInstalled() {
      if (installCheckStarted) return
      installCheckStarted = true

      let installed = ''
      try {
        installed = await invoke('get_config', { key: 'gameIsInstalled' })
      } catch (error) {
        console.error('读取配置失败:', error)
      }

      if (installed === 'false') {
        // 监听下载进度事件
        const unlisten = await listen('download-progress', (event) => {
          setDownloadPercent(event.payload.percent)
        })

        try {
          await invoke('download_game')
          setStatus('ready')
          setStatusText('游戏下载完成，可以开始冒险了')
          setDownloadPercent(100)
        } catch (error) {
          console.error('下载失败:', error)
          setStatus('ready')
          setStatusText('游戏下载失败，请检查网络后重试')
        } finally {
          unlisten()
        }
      } else {
        setStatus('ready')
        setStatusText('游戏下载完成，可以开始冒险了')
      }
    }
    ensureGameInstalled()
  }, [])

  function scrollChatToEnd() {
    requestAnimationFrame(() => {
      chatListRef.current?.scrollTo({ top: chatListRef.current.scrollHeight, behavior: 'smooth' })
    })
  }

  function openChat() {
    setShowChat(true)
    scrollChatToEnd()
  }

  async function sendMessage(message) {
    const content = (message ?? draftMessage).trim()
    if (!content || isReplying) return

    setChatMessages((prev) => [...prev, { id: Date.now(), role: 'user', text: content }])
    setDraftMessage('')
    setIsReplying(true)
    scrollChatToEnd()

    await new Promise((resolve) => window.setTimeout(resolve, 420))
    setChatMessages((prev) => [...prev, { id: Date.now() + 1, role: 'assistant', text: getAssistantReply(content) }])
    setIsReplying(false)
    scrollChatToEnd()
  }

  async function startGame() {
    if (isPlaying) {
      try {
        await invoke('close_game')
        setStatus('ready')
        setStatusText('准备开始冒险')
      } catch (error) {
        console.error('Close failed:', error)
        setStatusText('暂时无法结束游戏')
      }
      return
    }

    setStatusText('正在启动游戏')

    try {
      await invoke('launch_game')
      setStatus('playing')
      setStatusText('游戏正在运行')
    } catch (error) {
      console.error('Launch failed:', error)
      setStatusText('启动失败，请稍后重试')
    }
  }

  function handleComposerKeyDown(event) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      sendMessage()
    }
  }

  return (
    <div className="app-shell">
      <div className="scene" aria-hidden="true"></div>
      <div className="scene-overlay" aria-hidden="true"></div>
      <div className="scene-grid" aria-hidden="true"></div>

      <header className="topbar">
        <div className="brand-lockup">
          <div className="brand-mark" aria-hidden="true">
            <Box size={19} strokeWidth={2.3} />
          </div>
          <p>{gameInfo.name}</p>
        </div>
        <button className="assistant-trigger" type="button" title="打开游戏助手" onClick={openChat}>
          <span className="assistant-trigger-icon"><Bot size={19} strokeWidth={2.2} /></span>
          <span>游戏助手</span>
        </button>
      </header>

      <main className="launcher-main">
        <section className="hero-copy" aria-labelledby="launcher-title">
          <div className="eyebrow"><span></span> WELCOME BACK</div>
          <h1 id="launcher-title">{gameInfo.title}</h1>
          <p>{gameInfo.subtitle}</p>
        </section>

        <section className="launch-deck" aria-label="游戏启动">
          <div className="deck-intro">
            <div className="deck-symbol" aria-hidden="true">
              <Gamepad2 size={22} strokeWidth={2} />
            </div>
            <div>
              <p className="deck-state">{isDownloading ? '游戏下载中' : isPlaying ? '游戏正在运行' : '已准备就绪'}</p>
              <p className="deck-hint">{isDownloading ? `下载进度 ${downloadPercent.toFixed(1)}%` : isPlaying ? '愿你的冒险一切顺利' : '随时可以开始新的冒险'}</p>
            </div>
          </div>
          <h2>{isDownloading ? '正在准备游戏...' : isPlaying ? '愿你的冒险一切顺利' : '准备好出发了吗？'}</h2>
          <p className="deck-copy">
            {isDownloading ? '游戏下载完成后，即可开始冒险。' : isPlaying ? '游戏正在运行。' : '点击下方按钮，即刻进入游戏。'}
          </p>

          <div className="launch-divider"></div>

          <button
            className={`launch-button${isPlaying ? ' running' : ''}${isDownloading ? ' downloading' : ''}`}
            type="button"
            onClick={startGame}
            disabled={isDownloading}
            aria-busy={isDownloading}
          >
            {isDownloading && <DownloadWaveSvg percent={downloadPercent} />}
            <span className="launch-button-content">
              {isPlaying ? <Square size={18} fill="currentColor" /> : !isDownloading && <Play size={20} fill="currentColor" />}
              <span>{isDownloading ? '游戏下载中...' : isPlaying ? '结束游戏' : '启动游戏'}</span>
            </span>
          </button>
          <p className="launch-status">
            <span className={isPlaying ? 'running' : ''}></span>{statusText}
          </p>
        </section>
      </main>

      <footer className="footer-bar">© {new Date().getFullYear()} {gameInfo.name}</footer>

      {showChat && (
        <button className="chat-backdrop" type="button" aria-label="关闭游戏助手" onClick={() => setShowChat(false)}></button>
      )}

      {showChat && (
        <aside className="chat-panel" aria-label="游戏助手">
          <header className="chat-header">
            <div className="chat-title">
              <span className="chat-title-icon"><Bot size={19} strokeWidth={2.2} /></span>
              <div>
                <p>游戏助手</p>
                <span>随时问我游戏里的事</span>
              </div>
            </div>
            <button className="chat-icon-button" type="button" title="关闭游戏助手" aria-label="关闭游戏助手" onClick={() => setShowChat(false)}>
              <X size={18} />
            </button>
          </header>

          <div ref={chatListRef} className="chat-messages" aria-live="polite">
            {chatMessages.map((message) => (
              <div key={message.id} className={`chat-message ${message.role}`}>
                {message.role === 'assistant' && (
                  <span className="assistant-avatar"><Bot size={15} strokeWidth={2.2} /></span>
                )}
                <p>{message.text}</p>
              </div>
            ))}
            {isReplying && (
              <div className="chat-message assistant thinking" aria-label="游戏助手正在输入">
                <span className="assistant-avatar"><Bot size={15} strokeWidth={2.2} /></span>
                <p><i></i><i></i><i></i></p>
              </div>
            )}
          </div>

          {chatMessages.length === 1 && (
            <div className="quick-prompts" aria-label="推荐问题">
              {quickPrompts.map((prompt) => (
                <button key={prompt} type="button" onClick={() => sendMessage(prompt)}>
                  {prompt}
                </button>
              ))}
            </div>
          )}

          <form className="chat-composer" onSubmit={(e) => { e.preventDefault(); sendMessage() }}>
            <textarea
              rows="1"
              maxLength="300"
              placeholder="问问游戏里的事..."
              value={draftMessage}
              onChange={(e) => setDraftMessage(e.target.value)}
              onKeyDown={handleComposerKeyDown}
            ></textarea>
            <button type="submit" title="发送消息" aria-label="发送消息" disabled={!draftMessage.trim() || isReplying}>
              <SendHorizontal size={18} />
            </button>
          </form>
        </aside>
      )}
    </div>
  )
}
