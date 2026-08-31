import { create } from 'zustand'
import type { ViewMode } from '../types'

interface UiState {
  currentView: ViewMode
  showChat: boolean
  openChat: () => void
  closeChat: () => void
  toggleAdvanced: () => void
  setCurrentView: (view: ViewMode) => void
}

// 界面状态：当前视图（主界面/高级模式）与游戏助手面板开关
export const useUiStore = create<UiState>((set) => ({
  currentView: 'launcher',
  showChat: false,

  openChat: () => set({ showChat: true }),
  closeChat: () => set({ showChat: false }),
  toggleAdvanced: () =>
    set((s) => ({ currentView: s.currentView === 'advanced' ? 'launcher' : 'advanced' })),
  setCurrentView: (view) => set({ currentView: view }),
}))
