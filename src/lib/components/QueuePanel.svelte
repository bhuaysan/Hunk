<script lang="ts">
  import { basename, operationLabel } from '../presentation';
  import type { QueueItem } from '../types';

  export let items: QueueItem[] = [];
  export let onRemove: (id: string) => void;
</script>

<aside class="queue-panel" aria-labelledby="queue-title">
  <header>
    <div>
      <p class="section-label">Serial processing</p>
      <h2 id="queue-title">Queue</h2>
    </div>
    <span>{items.length.toString().padStart(2, '0')}</span>
  </header>
  {#if items.length === 0}
    <div class="queue-empty">
      <span aria-hidden="true">↳</span>
      <strong>No prepared jobs</strong>
      <p>Choose an action for a source. Jobs will run one at a time.</p>
    </div>
  {:else}
    <ol>
      {#each items as item, index (item.id)}
        <li class:blocked={item.status === 'blocked'}>
          <div class="queue-order">{(index + 1).toString().padStart(2, '0')}</div>
          <div class="queue-copy">
            <strong>{basename(item.source.primaryFile)}</strong>
            <span>{operationLabel(item.operation)}</span>
            <small>{item.message}</small>
          </div>
          <button
            type="button"
            onclick={() => onRemove(item.id)}
            aria-label={`Remove ${basename(item.source.primaryFile)} from queue`}>×</button
          >
        </li>
      {/each}
    </ol>
    <p class="queue-note">
      Prepared jobs are handed to the durable engine when processing is available.
    </p>
  {/if}
</aside>

<style>
  .queue-panel {
    min-width: 0;
    border-left: 1px solid var(--alloy);
    background: color-mix(in srgb, var(--surface) 56%, var(--porcelain));
  }
  header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    min-height: 72px;
    padding: 0 18px 16px;
    border-bottom: 1px solid var(--alloy);
  }
  h2 {
    margin: 0;
    font: 600 25px/1 var(--display);
  }
  header > span {
    color: var(--muted);
    font: 11px var(--mono);
  }
  .queue-empty {
    display: grid;
    justify-items: center;
    padding: 52px 24px;
    color: var(--muted);
    text-align: center;
  }
  .queue-empty > span {
    display: grid;
    width: 38px;
    height: 38px;
    place-items: center;
    margin-bottom: 12px;
    border: 1px solid var(--alloy);
    border-radius: 50%;
    color: var(--disc-blue);
    font: 17px var(--mono);
  }
  .queue-empty strong {
    color: var(--basalt);
    font-size: 12px;
  }
  .queue-empty p {
    max-width: 190px;
    margin: 6px 0 0;
    font-size: 10px;
    line-height: 1.5;
  }
  ol {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: grid;
    grid-template-columns: 25px minmax(0, 1fr) 22px;
    gap: 9px;
    padding: 15px 12px;
    border-bottom: 1px solid var(--alloy);
  }
  li.blocked {
    box-shadow: inset 3px 0 var(--danger);
  }
  .queue-order {
    color: var(--muted);
    font: 9px var(--mono);
  }
  .queue-copy {
    display: grid;
    min-width: 0;
    gap: 3px;
  }
  .queue-copy strong {
    overflow: hidden;
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .queue-copy span {
    color: var(--disc-blue);
    font-size: 10px;
    font-weight: 650;
  }
  .queue-copy small {
    color: var(--muted);
    font-size: 9px;
    line-height: 1.4;
  }
  li button {
    align-self: start;
    border: 0;
    color: var(--muted);
    background: transparent;
    cursor: pointer;
  }
  .queue-note {
    margin: 16px;
    color: var(--muted);
    font-size: 9px;
    line-height: 1.5;
  }
</style>
