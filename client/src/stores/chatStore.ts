import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { isTauri } from '../constants'
import { errorText } from '../utils'
import { useToastStore } from './toastStore'
import type { ChatMessage } from '../types'

interface ChatState {
  chatMessages: ChatMessage[]
  isReplying: boolean
  streamStarted: boolean
  // 记录当前流式输出的助手消息 id，用于逐字追加
  streamingAssistantId: number | null
  sendMessage: (message: string) => Promise<void>
}

const initialMessage: ChatMessage = {
  id: 1,
  role: 'assistant',
  text: '你好！我是游戏助手。想聊聊生存、合成、探索，还是和朋友一起玩？',
}

export const useChatStore = create<ChatState>((set, get) => ({
  chatMessages: [initialMessage],
  isReplying: false,
  streamStarted: false,
  streamingAssistantId: null,

  // 发送 AI 消息：输入框与消息列表由 ChatPanel 管理，这里负责流式接收与状态更新
  sendMessage: async (message: string) => {
    const { notify } = useToastStore.getState()
    if (!isTauri) {
      notify('浏览器预览模式，无法与 AI 对话', 'info')
      return
    }
    const content = message.trim()
    if (!content || get().isReplying) return

    set((s) => ({
      chatMessages: [...s.chatMessages, { id: Date.now(), role: 'user', text: content }],
      isReplying: true,
      streamStarted: false,
    }))

    let finished = false
    let unlistenChunk: (() => void) | undefined
    let unlistenDone: (() => void) | undefined

    const finish = () => {
      if (finished) return
      finished = true
      set({ isReplying: false, streamStarted: false, streamingAssistantId: null })
      unlistenChunk?.()
      unlistenDone?.()
    }

    // 流式监听：首块到达时创建助手消息，后续逐字追加
    unlistenChunk = await listen<string>('chat-chunk', (event) => {
      const chunk = event.payload
      const { streamingAssistantId, chatMessages } = get()
      if (streamingAssistantId == null) {
        const newId = Date.now()
        set({
          streamingAssistantId: newId,
          streamStarted: true,
          chatMessages: [...chatMessages, { id: newId, role: 'assistant', text: chunk }],
        })
      } else {
        set({
          chatMessages: chatMessages.map((m) =>
            m.id === streamingAssistantId ? { ...m, text: m.text + chunk } : m,
          ),
        })
      }
    })
    unlistenDone = await listen('chat-done', finish)

    try {
      await invoke('send_messages_to_mode', { message: content })
      finish()
    } catch (error) {
      console.error('AI 请求失败:', error)
      notify(`AI 请求失败：${errorText(error)}`, 'error')
      const { streamingAssistantId, chatMessages } = get()
      if (streamingAssistantId == null) {
        set({
          chatMessages: [
            ...chatMessages,
            { id: Date.now(), role: 'assistant', text: '抱歉，AI 暂时无法回答，请稍后重试。' },
          ],
        })
      } else {
        set({
          chatMessages: chatMessages.map((m) =>
            m.id === streamingAssistantId ? { ...m, text: m.text + '（请求中断，请重试）' } : m,
          ),
        })
      }
      finish()
    }
  },
}))
