<script>
  import { createEventDispatcher } from 'svelte'
  import { Play, Square } from 'lucide-svelte'

  export let isDownloading = false
  export let isPlaying = false
  export let needDownload = false
  export let isExtracting = false
  export let downloadPercent = 0

  const dispatch = createEventDispatcher()
</script>

<button
  class="launch-button"
  class:running={isPlaying || isExtracting}
  class:downloading={isDownloading && !isExtracting}
  type="button"
  on:click={() => dispatch('click')}
  disabled={isDownloading}
  aria-busy={isDownloading}
>
  {#if isDownloading}
    <span
      class="download-wave-svg"
      aria-hidden="true"
      style="animation: none; transform: translateY({100 - downloadPercent}%);"
    >
      <svg class="download-wave-layer download-wave-layer--back" viewBox="0 0 2000 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="download-wave-back" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#a5e8f8" stop-opacity="0.24" />
            <stop offset="100%" stop-color="#58c4e8" stop-opacity="0.26" />
          </linearGradient>
        </defs>
        <path d="M0 66 C83 59 167 59 250 66 C333 73 417 73 500 66 C583 59 667 59 750 66 C833 73 917 73 1000 66 C1083 59 1167 59 1250 66 C1333 73 1417 73 1500 66 C1583 59 1667 59 1750 66 C1833 73 1917 73 2000 66 V100 H0 Z" fill="url(#download-wave-back)" />
      </svg>
      <svg class="download-wave-layer download-wave-layer--mid" viewBox="0 0 2000 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="download-wave-mid" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#91e3f6" stop-opacity="0.36" />
            <stop offset="100%" stop-color="#58c4e8" stop-opacity="0.4" />
          </linearGradient>
        </defs>
        <path d="M0 64 C83 55 167 55 250 64 C333 73 417 73 500 64 C583 55 667 55 750 64 C833 73 917 73 1000 64 C1083 55 1167 55 1250 64 C1333 73 1417 73 1500 64 C1583 55 1667 55 1750 64 C1833 73 1917 73 2000 64 V100 H0 Z" fill="url(#download-wave-mid)" />
      </svg>
      <svg class="download-wave-layer download-wave-layer--front" viewBox="0 0 2000 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="download-wave-front" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#c7f5ff" stop-opacity="0.7" />
            <stop offset="58%" stop-color="#74d3ee" stop-opacity="0.74" />
            <stop offset="100%" stop-color="#58c4e8" stop-opacity="0.78" />
          </linearGradient>
        </defs>
        <path d="M0 62 C83 52 167 52 250 62 C333 72 417 72 500 62 C583 52 667 52 750 62 C833 72 917 72 1000 62 C1083 52 1167 52 1250 62 C1333 72 1417 72 1500 62 C1583 52 1667 52 1750 62 C1833 72 1917 72 2000 62 V100 H0 Z" fill="url(#download-wave-front)" />
        <path d="M0 62 C83 52 167 52 250 62 C333 72 417 72 500 62 C583 52 667 52 750 62 C833 72 917 72 1000 62 C1083 52 1167 52 1250 62 C1333 72 1417 72 1500 62 C1583 52 1667 52 1750 62 C1833 72 1917 72 2000 62" class="download-wave-crest" />
      </svg>
    </span>
  {/if}
  <span class="launch-button-content">
    {#if isPlaying}
      <Square size={18} fill="currentColor" />
    {:else if !isDownloading}
      <Play size={20} fill="currentColor" />
    {/if}
    <span>{isExtracting ? '正在解压...' : isDownloading ? '游戏下载中...' : needDownload ? '下载游戏' : isPlaying ? '结束游戏' : '启动游戏'}</span>
  </span>
</button>
