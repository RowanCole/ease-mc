// 通用工具函数

// 将 Tauri / 运行时抛出的错误统一转为可读文本
export function errorText(error: unknown): string {
  return typeof error === 'string' ? error : error instanceof Error ? error.message : String(error)
}
