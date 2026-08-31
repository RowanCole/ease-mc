import Scene from './Scene'
import HeroSection from './HeroSection'
import LaunchCard from './LaunchCard'
import { gameInfo } from '../constants'

// 普通模式页面：背景场景 + 欢迎区 + 启动卡片
export default function LauncherView() {
  return (
    <>
      <Scene />
      <main className="launcher-main">
        <HeroSection title={gameInfo.title} subtitle={gameInfo.subtitle} />
        <LaunchCard />
      </main>
    </>
  )
}
