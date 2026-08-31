import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import './App.css'
import { gameInfo, isTauri } from './constants'
import { useUiStore } from './stores/uiStore'
import { useGameStore } from './stores/gameStore'
import TopBar from './components/TopBar'
import LauncherView from './components/LauncherView'
import AdvancedMode from './components/AdvancedMode'
import ChatPanel from './components/ChatPanel'
import ToastStack from './components/ToastStack'

export default function App() {
  const currentView = useUiStore((s) => s.currentView)
  const showChat = useUiStore((s) => s.showChat)
  const ensureGameInstalled = useGameStore((s) => s.ensureGameInstalled)
  const handleGameExited = useGameStore((s) => s.handleGameExited)

  useEffect(() => {
    let unlistenExited: (() => void) | undefined
    let mounted = true

    if (isTauri) {
      // 游戏进程被手动关闭时，恢复启动器为可开始状态
      listen('game-exited', handleGameExited).then((unlisten) => {
        if (mounted) {
          unlistenExited = unlisten
        } else {
          unlisten()
        }
      })
    }
    void ensureGameInstalled()

    return () => {
      mounted = false
      unlistenExited?.()
    }
  }, [ensureGameInstalled, handleGameExited])

  return (
    <div className={`app-shell${currentView === 'advanced' ? ' advanced' : ''}`}>
      <TopBar gameName={gameInfo.name} />

      {currentView === 'advanced' ? (
        <div key="advanced" className="view-layer view-layer--advanced">
          <AdvancedMode />
        </div>
      ) : (
        <div key="launcher" className="view-layer view-layer--launcher">
          <LauncherView />
        </div>
      )}

      {showChat && <ChatPanel />}

      <ToastStack />
    </div>
  )
}
