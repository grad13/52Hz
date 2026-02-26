<script lang="ts">
  let { phaseLabel, remaining, paused }: {
    phaseLabel: string;
    remaining: string;
    paused: boolean;
  } = $props();
</script>

<section class="status">
  <div class="phase">{phaseLabel}</div>
  <div class="time">{remaining}</div>
  {#if paused}
    <div class="paused-badge">一時停止中</div>
  {/if}
</section>

<style>
  .status {
    text-align: center;
    padding: 0.8rem 0;
  }

  .phase {
    font-size: 0.85rem;
    color: var(--text-secondary);
    margin-bottom: 0.3rem;
  }

  .time {
    font-size: 2.8rem;
    font-weight: 300;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.05em;
  }

  .paused-badge {
    display: inline-block;
    margin-top: 0.4rem;
    padding: 0.15rem 0.6rem;
    font-size: 0.75rem;
    background: var(--danger);
    border-radius: 4px;
    color: #fff;
  }
</style>
