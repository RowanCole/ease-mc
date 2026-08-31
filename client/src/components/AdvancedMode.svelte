<script>
  import { createEventDispatcher } from 'svelte'
  import { ArrowLeft, SlidersHorizontal } from 'lucide-svelte'

  const dispatch = createEventDispatcher()
</script>

<!-- 高级模式页面背景：亮色蓝白主题，浅蓝渐变 + 点阵网格（待开发，可替换为专属背景图） -->
<div class="advanced-scene" aria-hidden="true"></div>

<main class="advanced-page">
  <section class="advanced-panel" aria-label="高级模式">
    <div class="advanced-symbol" aria-hidden="true">
      <SlidersHorizontal size={22} strokeWidth={2.2} />
    </div>
    <p class="advanced-eyebrow">ADVANCED MODE</p>
    <h2>高级模式</h2>
    <p class="advanced-copy">该操作界面正在开发中，敬请期待。</p>
    <button class="advanced-back" type="button" on:click={() => dispatch('back')}>
      <ArrowLeft size={16} strokeWidth={2.3} />
      <span>返回主界面</span>
    </button>
  </section>
</main>
