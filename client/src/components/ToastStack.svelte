<script>
  import { createEventDispatcher } from 'svelte'
  import { fly } from 'svelte/transition'
  import { X } from 'lucide-svelte'

  export let toasts = []

  const dispatch = createEventDispatcher()
</script>

<div class="toast-stack" aria-live="polite">
  {#each toasts as toast (toast.id)}
    <div
      class="toast toast--{toast.type}"
      role="alert"
      transition:fly={{ y: -12, duration: 260 }}
    >
      <p class="toast-message">{toast.message}</p>
      <button type="button" title="关闭" aria-label="关闭通知" on:click={() => dispatch('dismiss', toast.id)}>
        <X size={14} />
      </button>
    </div>
  {/each}
</div>
