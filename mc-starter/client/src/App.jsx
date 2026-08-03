import { useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Bot, Box, Gamepad2, Play, SendHorizontal, Square, X } from 'lucide-react'
import './App.css'

const gameInfo = {
  name: 'MC STARTER',
  title: 'Minecraft',
  subtitle: '进入属于你的方块世界',
}

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

export default function App() {
  const [status, setStatus] = useState('ready')
  const [statusText, setStatusText] = useState('准备开始冒险')
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
              <p className="deck-state">{isPlaying ? '游戏正在运行' : '已准备就绪'}</p>
              <p className="deck-hint">{isPlaying ? '愿你的冒险一切顺利' : '随时可以开始新的冒险'}</p>
            </div>
          </div>
          <h2>{isPlaying ? '愿你的冒险一切顺利' : '准备好出发了吗？'}</h2>
          <p className="deck-copy">
            {isPlaying ? '游戏正在运行。' : '点击下方按钮，即刻进入游戏。'}
          </p>

          <div className="launch-divider"></div>

          <button className={`launch-button${isPlaying ? ' running' : ''}`} type="button" onClick={startGame}>
            {isPlaying ? <Square size={18} fill="currentColor" /> : <Play size={20} fill="currentColor" />}
            <span>{isPlaying ? '结束游戏' : '开始游戏'}</span>
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
