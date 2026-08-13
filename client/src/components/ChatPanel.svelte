<script>
  import { afterUpdate, createEventDispatcher } from 'svelte'
  import { Bot, SendHorizontal, X } from 'lucide-svelte'
  import { marked } from 'marked'
  import { quickPrompts } from '../constants.js'

  // 保留 AI 回复中的换行（流式逐字追加依赖换行渲染）
  marked.setOptions({ breaks: true })

  export let chatMessages = []
  export let isReplying = false
  export let streamStarted = false

  const dispatch = createEventDispatcher()

  let draftMessage = ''
  let chatListEl

  // 消息变化（流式逐字追加等）时自动滚动到底部
  afterUpdate(() => {
    chatListEl?.scrollTo({ top: chatListEl.scrollHeight, behavior: 'smooth' })
  })

  function submit() {
    const content = draftMessage.trim()
    if (!content || isReplying) return
    dispatch('send', content)
    draftMessage = ''
  }

  function handleKeyDown(event) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      submit()
    }
  }
</script>

<button class="chat-backdrop" type="button" aria-label="关闭游戏助手" on:click={() => dispatch('close')}></button>

<aside class="chat-panel" aria-label="游戏助手">
  <header class="chat-header">
    <div class="chat-title">
      <span class="chat-title-icon"><Bot size={19} strokeWidth={2.2} /></span>
      <div>
        <p>游戏助手</p>
        <span>随时问我游戏里的事</span>
      </div>
    </div>
    <button class="chat-icon-button" type="button" title="关闭游戏助手" aria-label="关闭游戏助手" on:click={() => dispatch('close')}>
      <X size={18} />
    </button>
  </header>

  <div bind:this={chatListEl} class="chat-messages" aria-live="polite">
    {#each chatMessages as message (message.id)}
      <div class="chat-message {message.role}">
        {#if message.role === 'assistant'}
          <span class="assistant-avatar"><Bot size={15} strokeWidth={2.2} /></span>
          <div class="chat-bubble markdown">{@html marked.parse(message.text)}</div>
        {:else}
          <p>{message.text}</p>
        {/if}
      </div>
    {/each}
    {#if isReplying && !streamStarted}
      <div class="chat-message assistant thinking" aria-label="游戏助手正在输入">
        <span class="assistant-avatar"><Bot size={15} strokeWidth={2.2} /></span>
        <p><i></i><i></i><i></i></p>
      </div>
    {/if}
  </div>

  {#if chatMessages.length === 1}
    <div class="quick-prompts" aria-label="推荐问题">
      {#each quickPrompts as prompt (prompt)}
        <button type="button" on:click={() => dispatch('send', prompt)}>{prompt}</button>
      {/each}
    </div>
  {/if}

  <form class="chat-composer" on:submit|preventDefault={submit}>
    <textarea
      rows="1"
      maxlength="300"
      placeholder="问问游戏里的事..."
      bind:value={draftMessage}
      on:keydown={handleKeyDown}
    ></textarea>
    <button type="submit" title="发送消息" aria-label="发送消息" disabled={!draftMessage.trim() || isReplying}>
      <SendHorizontal size={18} />
    </button>
  </form>
</aside>
