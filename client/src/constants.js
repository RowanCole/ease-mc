// 全局常量与工具

export const gameInfo = {
  name: '让Minecraft更方便',
  title: 'Minecraft',
  subtitle: '进入属于你的方块世界',
}

export const quickPrompts = ['新手应该先做什么？', '怎么找到钻石？', '下界要注意什么？']

// 是否运行在 Tauri 环境（浏览器预览模式用于纯 UI 调试）
export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
