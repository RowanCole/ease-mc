import { create } from 'zustand'
import type { Toast, ToastType } from '../types'

const MAX_TOASTS = 3
const TOAST_DURATION = 5000

interface ToastState {
  toasts: Toast[]
  notify: (message: string, type?: ToastType) => void
  dismiss: (id: number) => void
}

let toastId = 0
// 记录每个弹窗的自动关闭定时器，手动关闭时一并清理
const timers = new Map<number, ReturnType<typeof setTimeout>>()

export const useToastStore = create<ToastState>((set, get) => ({
  toasts: [],

  notify: (message, type = 'error') => {
    const { toasts, dismiss } = get()
    // 相同消息已在展示中则忽略，避免重复点击导致弹窗堆积
    if (toasts.some((t) => t.message === message && t.type === type)) return

    const id = ++toastId
    const next = [...toasts, { id, message, type }]
    set({ toasts: next.length > MAX_TOASTS ? next.slice(next.length - MAX_TOASTS) : next })

    const timer = setTimeout(() => dismiss(id), TOAST_DURATION)
    timers.set(id, timer)
  },

  dismiss: (id) => {
    const timer = timers.get(id)
    if (timer) {
      clearTimeout(timer)
      timers.delete(id)
    }
    set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }))
  },
}))
