// 全局共享类型定义

export type GameStatus = 'idle' | 'ready' | 'downloading' | 'playing'

export type ToastType = 'error' | 'success' | 'info'

export type ViewMode = 'launcher' | 'advanced'

export type MessageRole = 'user' | 'assistant'

export interface ChatMessage {
  id: number
  role: MessageRole
  text: string
}

export interface Toast {
  id: number
  message: string
  type: ToastType
}

export interface GameInfo {
  name: string
  title: string
  subtitle: string
}
