<script lang="ts">
  import { jobStateLabel, localizeJobMessage, operationLabel, t, type Locale } from '../i18n';
  import { basename } from '../presentation';
  import type { JobRecord } from '../types';
  import TrackBand from './TrackBand.svelte';

  export let items: JobRecord[] = [];
  export let paused = false;
  export let activeJobId: string | null = null;
  export let locale: Locale;
  export let inactive = false;
  export let onPause: (paused: boolean) => void;
  export let onCancel: (id: string) => void;
  export let onRetry: (id: string) => void;
  export let onRemove: (id: string) => void;

  const cancellable = (item: JobRecord) =>
    ['queued', 'preflight', 'running', 'verifying', 'blocked'].includes(item.state);
</script>

<aside class="queue-panel" aria-labelledby="queue-title" inert={inactive}>
  <header>
    <div>
      <p class="section-label">{t(locale, 'serialProcessing')}</p>
      <h2 id="queue-title">{t(locale, 'queue')}</h2>
    </div>
    <div class="queue-tools">
      <span>{items.length.toString().padStart(2, '0')}</span>
      <button type="button" onclick={() => onPause(!paused)} disabled={!items.length}>
        {paused ? t(locale, 'resume') : t(locale, 'pause')}
      </button>
    </div>
  </header>
  {#if items.length === 0}
    <div class="queue-empty">
      <span aria-hidden="true">↳</span>
      <strong>{t(locale, 'noQueuedJobs')}</strong>
      <p>{t(locale, 'queueEmptyExplanation')}</p>
    </div>
  {:else}
    {#if paused}<p class="paused-note" role="status">
        {t(locale, 'queuePaused')}
      </p>{/if}
    <ol>
      {#each items as item, index (item.id)}
        <li class:active={activeJobId === item.id} class:blocked={item.state === 'blocked'}>
          <div class="queue-order">{(index + 1).toString().padStart(2, '0')}</div>
          <div class="queue-copy">
            <strong>{basename(item.spec.source.primaryFile)}</strong>
            <span
              >{operationLabel(locale, item.spec.operation)} · {jobStateLabel(
                locale,
                item.state,
              )}</span
            >
            <TrackBand
              tracks={item.spec.source.tracks}
              progress={item.progress?.percentage ?? null}
              {locale}
              compact
            />
            <small class:error={Boolean(item.error)}
              >{item.error ?? localizeJobMessage(locale, item.message)}</small
            >
          </div>
          <div class="item-actions">
            {#if item.state === 'blocked'}
              <button
                type="button"
                onclick={() => onRetry(item.id)}
                aria-label={t(locale, 'retryJob')}>↻</button
              >
            {/if}
            {#if cancellable(item)}
              <button
                type="button"
                onclick={() => onCancel(item.id)}
                aria-label={t(locale, 'cancelJob')}>×</button
              >
            {:else}
              <button
                type="button"
                onclick={() => onRemove(item.id)}
                aria-label={t(locale, 'removeJob')}>×</button
              >
            {/if}
          </div>
        </li>
      {/each}
    </ol>
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
  .queue-tools {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .queue-tools span {
    color: var(--muted);
    font: 11px var(--mono);
  }
  .queue-tools button,
  .item-actions button {
    min-width: 28px;
    min-height: 28px;
    padding: 4px 6px;
    border: 1px solid var(--alloy);
    border-radius: 4px;
    color: var(--interactive);
    background: transparent;
    font-size: 9px;
    font-weight: 700;
    cursor: pointer;
  }
  .paused-note {
    margin: 0;
    padding: 9px 14px;
    border-bottom: 1px solid var(--alloy);
    color: var(--warning-text);
    font-size: 9px;
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
    color: var(--interactive);
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
    grid-template-columns: 25px minmax(0, 1fr) auto;
    gap: 9px;
    padding: 15px 12px;
    border-bottom: 1px solid var(--alloy);
  }
  li.active {
    box-shadow: inset 3px 0 var(--disc-blue);
    background: color-mix(in srgb, var(--disc-blue) 6%, transparent);
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
    gap: 5px;
  }
  .queue-copy strong {
    overflow: hidden;
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .queue-copy > span {
    color: var(--interactive);
    font-size: 9px;
    font-weight: 650;
    text-transform: capitalize;
  }
  .queue-copy small {
    color: var(--muted);
    font-size: 9px;
    line-height: 1.4;
  }
  .queue-copy small.error {
    color: var(--danger-text);
  }
  .item-actions {
    display: flex;
    gap: 4px;
    align-items: start;
  }
</style>
