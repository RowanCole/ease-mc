<script>
  import { createEventDispatcher } from 'svelte'
  import { Bot, Box } from 'lucide-svelte'

  export let gameName

  const dispatch = createEventDispatcher()
</script>

<header class="topbar">
  <div class="brand-lockup">
    <div class="brand-mark" aria-hidden="true">
      <Box size={19} strokeWidth={2.3} />
    </div>
    <p>{gameName}</p>
  </div>
  <button class="assistant-trigger" type="button" title="打开游戏助手" on:click={() => dispatch('openAssistant')}>
    <span class="assistant-trigger-icon"><Bot size={19} strokeWidth={2.2} /></span>
    <span>游戏助手</span>
  </button>
</header>
