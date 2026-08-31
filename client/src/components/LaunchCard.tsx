import { Gamepad2 } from 'lucide-react'
import LaunchButton from './LaunchButton'
import { useGameStore } from '../stores/gameStore'

export default function LaunchCard() {
  const status = useGameStore((s) => s.status)
  const downloadPercent = useGameStore((s) => s.downloadPercent)
  const needDownload = useGameStore((s) => s.needDownload)
  const statusText = useGameStore((s) => s.statusText)

  const isDownloading = status === 'downloading'
  const isPlaying = status === 'playing'

  const stateLabel = isDownloading
    ? '游戏下载中'
    : isPlaying
      ? '游戏正在运行'
      : needDownload
        ? '游戏未安装'
        : '已准备就绪'
  const hintLabel = isDownloading
    ? `下载进度 ${downloadPercent.toFixed(1)}%`
    : isPlaying
      ? '愿你的冒险一切顺利'
      : needDownload
        ? '游戏尚未安装'
        : '随时可以开始新的冒险'
  const title = isDownloading
    ? '正在准备游戏...'
    : isPlaying
      ? '愿你的冒险一切顺利'
      : needDownload
        ? '游戏尚未安装'
        : '准备好出发了吗？'
  const copy = isDownloading
    ? '游戏下载完成后，即可开始冒险。'
    : isPlaying
      ? '游戏正在运行。'
      : needDownload
        ? '点击下方按钮，下载并安装游戏。'
        : '点击下方按钮，即刻进入游戏。'

  return (
    <section className="launch-deck" aria-label="游戏启动">
      <div className="deck-intro">
        <div className="deck-symbol" aria-hidden="true">
          <Gamepad2 size={22} strokeWidth={2} />
        </div>
        <div>
          <p className="deck-state">{stateLabel}</p>
          <p className="deck-hint">{hintLabel}</p>
        </div>
      </div>
      <h2>{title}</h2>
      <p className="deck-copy">{copy}</p>

      <div className="launch-divider" />

      <LaunchButton />
      <p className="launch-status">
        <span className={isPlaying ? 'running' : ''} />
        {statusText}
      </p>
    </section>
  )
}
