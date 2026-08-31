// 背景场景：纯装饰层
export default function Scene() {
  return (
    <>
      <div className="scene" aria-hidden="true" />
      <div className="scene-overlay" aria-hidden="true" />
      <div className="scene-grid" aria-hidden="true" />
    </>
  )
}
