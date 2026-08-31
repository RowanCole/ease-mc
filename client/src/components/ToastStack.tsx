import { X } from 'lucide-react'
import { useToastStore } from '../stores/toastStore'

export default function ToastStack() {
  const toasts = useToastStore((s) => s.toasts)
  const dismiss = useToastStore((s) => s.dismiss)

  return (
    <div className="toast-stack" aria-live="polite">
      {toasts.map((toast) => (
        <div key={toast.id} className={`toast toast--${toast.type}`} role="alert">
          <p className="toast-message">{toast.message}</p>
          <button
            type="button"
            title="关闭"
            aria-label="关闭通知"
            onClick={() => dismiss(toast.id)}
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  )
}
