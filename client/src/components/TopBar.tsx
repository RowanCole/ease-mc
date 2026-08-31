import { Box, Bot } from 'lucide-react'
import { useUiStore } from '../stores/uiStore'

interface TopBarProps {
  gameName: string
}

export default function TopBar({ gameName }: TopBarProps) {
  const toggleAdvanced = useUiStore((s) => s.toggleAdvanced)
  const openChat = useUiStore((s) => s.openChat)

  return (
    <header className="topbar">
      <div className="brand-lockup">
        <button
          className="brand-mark"
          type="button"
          title="打开高级模式"
          aria-label="打开高级模式"
          onClick={toggleAdvanced}
        >
          <Box size={19} strokeWidth={2.3} />
        </button>
        <p>{gameName}</p>
      </div>
      <button
        className="assistant-trigger"
        type="button"
        title="打开游戏助手"
        onClick={openChat}
      >
        <span className="assistant-trigger-icon">
          <Bot size={19} strokeWidth={2.2} />
        </span>
        <span>游戏助手</span>
      </button>
    </header>
  )
}
