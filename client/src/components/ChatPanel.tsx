import { useEffect, useRef, useState } from 'react'
import { Bot, SendHorizontal, X } from 'lucide-react'
import { marked } from 'marked'
import { quickPrompts } from '../constants'
import { useChatStore } from '../stores/chatStore'
import { useUiStore } from '../stores/uiStore'
import type { FormEvent, KeyboardEvent } from 'react'

// 保留 AI 回复中的换行（流式逐字追加依赖换行渲染）
marked.setOptions({ breaks: true })

// marked 在同步配置下返回字符串，此处做类型收窄避免断言
function renderMarkdown(text: string): string {
  const html = marked.parse(text)
  return typeof html === 'string' ? html : ''
}

export default function ChatPanel() {
  const chatMessages = useChatStore((s) => s.chatMessages)
  const isReplying = useChatStore((s) => s.isReplying)
  const streamStarted = useChatStore((s) => s.streamStarted)
  const sendMessage = useChatStore((s) => s.sendMessage)
  const closeChat = useUiStore((s) => s.closeChat)

  const [draftMessage, setDraftMessage] = useState('')
  const chatListRef = useRef<HTMLDivElement>(null)

  // 消息变化（流式逐字追加等）时自动滚动到底部
  useEffect(() => {
    chatListRef.current?.scrollTo({ top: chatListRef.current.scrollHeight, behavior: 'smooth' })
  }, [chatMessages])

  const submit = () => {
    const content = draftMessage.trim()
    if (!content || isReplying) return
    void sendMessage(content)
    setDraftMessage('')
  }

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault()
    submit()
  }

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      submit()
    }
  }

  return (
    <>
      <button
        className="chat-backdrop"
        type="button"
        aria-label="关闭游戏助手"
        onClick={closeChat}
      />

      <aside className="chat-panel" aria-label="游戏助手">
        <header className="chat-header">
          <div className="chat-title">
            <span className="chat-title-icon">
              <Bot size={19} strokeWidth={2.2} />
            </span>
            <div>
              <p>游戏助手</p>
              <span>随时问我游戏里的事</span>
            </div>
          </div>
          <button
            className="chat-icon-button"
            type="button"
            title="关闭游戏助手"
            aria-label="关闭游戏助手"
            onClick={closeChat}
          >
            <X size={18} />
          </button>
        </header>

        <div ref={chatListRef} className="chat-messages" aria-live="polite">
          {chatMessages.map((message) => (
            <div key={message.id} className={`chat-message ${message.role}`}>
              {message.role === 'assistant' ? (
                <>
                  <span className="assistant-avatar">
                    <Bot size={15} strokeWidth={2.2} />
                  </span>
                  <div
                    className="chat-bubble markdown"
                    dangerouslySetInnerHTML={{ __html: renderMarkdown(message.text) }}
                  />
                </>
              ) : (
                <p>{message.text}</p>
              )}
            </div>
          ))}
          {isReplying && !streamStarted && (
            <div className="chat-message assistant thinking" aria-label="游戏助手正在输入">
              <span className="assistant-avatar">
                <Bot size={15} strokeWidth={2.2} />
              </span>
              <p>
                <i />
                <i />
                <i />
              </p>
            </div>
          )}
        </div>

        {chatMessages.length === 1 && (
          <div className="quick-prompts" aria-label="推荐问题">
            {quickPrompts.map((prompt) => (
              <button key={prompt} type="button" onClick={() => void sendMessage(prompt)}>
                {prompt}
              </button>
            ))}
          </div>
        )}

        <form className="chat-composer" onSubmit={handleSubmit}>
          <textarea
            rows={1}
            maxLength={300}
            placeholder="问问游戏里的事..."
            value={draftMessage}
            onChange={(e) => setDraftMessage(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          <button
            type="submit"
            title="发送消息"
            aria-label="发送消息"
            disabled={!draftMessage.trim() || isReplying}
          >
            <SendHorizontal size={18} />
          </button>
        </form>
      </aside>
    </>
  )
}
