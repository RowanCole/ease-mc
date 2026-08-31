<script>
  import Scene from './Scene.svelte'
  import HeroSection from './HeroSection.svelte'
  import LaunchCard from './LaunchCard.svelte'

  export let title
  export let subtitle
  export let isDownloading = false
  export let isPlaying = false
  export let needDownload = false
  export let downloadPercent = 0
  export let isExtracting = false
  export let statusText = ''
</script>

<!-- 普通模式页面：背景场景 + 欢迎区 + 启动卡片 -->
<Scene />

<main class="launcher-main">
  <HeroSection {title} {subtitle} />
  <LaunchCard
    {isDownloading}
    {isPlaying}
    {needDownload}
    {downloadPercent}
    {isExtracting}
    {statusText}
    on:action
  />
</main>
