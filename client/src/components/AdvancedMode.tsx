import { ArrowLeft, SlidersHorizontal } from 'lucide-react'
import { useUiStore } from '../stores/uiStore'

export default function AdvancedMode() {
  const setCurrentView = useUiStore((s) => s.setCurrentView)

  return (
    <>
      {/* 高级模式页面背景：亮色蓝白主题，浅蓝渐变 + 点阵网格（待开发，可替换为专属背景图） */}
      <div className="advanced-scene" aria-hidden="true" />
      <main className="advanced-page">
        <section className="advanced-panel" aria-label="高级模式">
          <div className="advanced-symbol" aria-hidden="true">
            <SlidersHorizontal size={22} strokeWidth={2.2} />
          </div>
          <p className="advanced-eyebrow">ADVANCED MODE</p>
          <h2>高级模式</h2>
          <p className="advanced-copy">该操作界面正在开发中，敬请期待。</p>
          <button className="advanced-back" type="button" onClick={() => setCurrentView('launcher')}>
            <ArrowLeft size={16} strokeWidth={2.3} />
            <span>返回主界面</span>
          </button>
        </section>
      </main>
    </>
  )
}
