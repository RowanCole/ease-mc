import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { isTauri } from '../constants'
import { errorText } from '../utils'
import { useToastStore } from './toastStore'
import type { GameStatus } from '../types'

interface GameState {
  status: GameStatus
  statusText: string
  downloadPercent: number
  isExtracting: boolean
  needDownload: boolean
  downloadGame: () => Promise<void>
  ensureGameInstalled: () => Promise<void>
  startGame: () => Promise<void>
  handleGameExited: () => void
}

export const useGameStore = create<GameState>((set, get) => ({
  status: 'idle',
  statusText: '正在检查游戏状态...',
  downloadPercent: 0,
  isExtracting: false,
  needDownload: false,

  // 下载游戏，启动检查与手动重试共用
  downloadGame: async () => {
    const { notify } = useToastStore.getState()
    if (!isTauri) {
      notify('浏览器预览模式，无法执行下载', 'info')
      return
    }
    // 防重入：下载/安装过程中忽略重复点击
    if (get().status === 'downloading' || get().isExtracting) {
      notify('游戏正在下载/安装中，请稍候', 'info')
      return
    }
    set({ status: 'downloading', statusText: '游戏下载中...', downloadPercent: 0 })

    const unlistenProgress = await listen<{ percent: number }>('download-progress', (event) => {
      set({ downloadPercent: event.payload.percent })
    })
    // 监听运行环境安装事件（JRE 下载解压阶段），切换按钮文案与颜色
    const unlistenExtract = await listen('extract-start', () => {
      set({ isExtracting: true, statusText: '正在安装运行环境...' })
      notify('正在安装运行环境，请稍候...', 'info')
    })

    try {
      await invoke('download_game')
      set({
        needDownload: false,
        status: 'ready',
        statusText: '游戏下载完成，可以开始冒险了',
        downloadPercent: 100,
      })
      notify('游戏下载完成，可以开始冒险了', 'success')
    } catch (error) {
      console.error('下载失败:', error)
      notify(`游戏下载失败：${errorText(error)}`, 'error')
      set({ needDownload: true, status: 'ready', statusText: '游戏下载失败，请检查网络后重试' })
    } finally {
      set({ isExtracting: false })
      unlistenProgress()
      unlistenExtract()
    }
  },

  // 应用启动时检查游戏是否已安装，未安装则等待用户点击「下载游戏」按钮触发下载
  ensureGameInstalled: async () => {
    if (!isTauri) {
      set({ status: 'ready', statusText: '浏览器预览模式（仅用于界面调试）' })
      return
    }
    let installed = ''
    try {
      installed = await invoke<string>('get_config', { key: 'gameIsInstalled' })
    } catch (error) {
      console.error('读取配置失败:', error)
    }

    if (installed === 'false') {
      set({ needDownload: true, status: 'ready', statusText: '游戏尚未安装，点击下方按钮开始下载' })
    } else {
      set({ status: 'ready', statusText: '游戏下载完成，可以开始冒险了' })
    }
  },

  startGame: async () => {
    const { notify } = useToastStore.getState()
    if (!isTauri) {
      notify('浏览器预览模式，无法启动游戏', 'info')
      return
    }
    // 游戏运行中则结束游戏
    if (get().status === 'playing') {
      try {
        await invoke('close_game')
        set({ status: 'ready', statusText: '准备开始冒险' })
      } catch (error) {
        console.error('Close failed:', error)
        set({ statusText: '暂时无法结束游戏' })
        notify(`无法结束游戏：${errorText(error)}`, 'error')
      }
      return
    }

    // 未安装则先下载
    if (get().needDownload) {
      await get().downloadGame()
      return
    }

    set({ statusText: '正在启动游戏' })
    try {
      await invoke('launch_game')
      set({ status: 'playing', statusText: '游戏正在运行' })
    } catch (error) {
      console.error('Launch failed:', error)
      set({ statusText: '启动失败，请稍后重试' })
      notify(`游戏启动失败：${errorText(error)}`, 'error')
    }
  },

  // 游戏进程被手动关闭时，恢复启动器为可开始状态
  handleGameExited: () => {
    set({ status: 'ready', statusText: '准备开始冒险' })
  },
}))
