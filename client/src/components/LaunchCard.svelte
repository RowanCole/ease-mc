<script>
  import { createEventDispatcher } from 'svelte'
  import { Gamepad2 } from 'lucide-svelte'
  import LaunchButton from './LaunchButton.svelte'

  export let isDownloading = false
  export let isPlaying = false
  export let needDownload = false
  export let downloadPercent = 0
  export let isExtracting = false
  export let statusText = ''

  const dispatch = createEventDispatcher()
</script>

<section class="launch-deck" aria-label="游戏启动">
  <div class="deck-intro">
    <div class="deck-symbol" aria-hidden="true">
      <Gamepad2 size={22} strokeWidth={2} />
    </div>
    <div>
      <p class="deck-state">{isDownloading ? '游戏下载中' : isPlaying ? '游戏正在运行' : needDownload ? '游戏未安装' : '已准备就绪'}</p>
      <p class="deck-hint">{isDownloading ? `下载进度 ${downloadPercent.toFixed(1)}%` : isPlaying ? '愿你的冒险一切顺利' : needDownload ? '游戏尚未安装' : '随时可以开始新的冒险'}</p>
    </div>
  </div>
  <h2>{isDownloading ? '正在准备游戏...' : isPlaying ? '愿你的冒险一切顺利' : needDownload ? '游戏尚未安装' : '准备好出发了吗？'}</h2>
  <p class="deck-copy">
    {isDownloading ? '游戏下载完成后，即可开始冒险。' : isPlaying ? '游戏正在运行。' : needDownload ? '点击下方按钮，下载并安装游戏。' : '点击下方按钮，即刻进入游戏。'}
  </p>

  <div class="launch-divider"></div>

  <LaunchButton
    {isDownloading}
    {isPlaying}
    {needDownload}
    {isExtracting}
    {downloadPercent}
    on:click={() => dispatch('action')}
  />
  <p class="launch-status">
    <span class:running={isPlaying}></span>{statusText}
  </p>
</section>
