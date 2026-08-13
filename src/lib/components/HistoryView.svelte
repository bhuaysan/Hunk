<script lang="ts">
  import { basename, formatBytes, operationLabel } from '../presentation';
  import type { JobRecord } from '../types';

  export let items: JobRecord[] = [];
  export let onWorkbench: () => void;
  export let onRetry: (id: string) => void;
  export let onRemove: (id: string) => void;

  const date = (timestamp: number | null) =>
    timestamp === null ? '—' : new Date(timestamp).toLocaleString();
</script>

<section class="history-view" aria-labelledby="history-title">
  <header class="history-heading">
    <div>
      <p class="section-label">Latest 100 jobs</p>
      <h1 id="history-title">History</h1>
    </div>
    <span>{items.length} records</span>
  </header>
  {#if items.length === 0}
    <div class="history-empty">
      <div aria-hidden="true">⌁</div>
      <h2>Completed work will collect here.</h2>
      <p>
        Each record includes paths, size savings, timestamps, status, and a bounded process log.
      </p>
      <button type="button" class="primary" onclick={onWorkbench}>Open workbench</button>
    </div>
  {:else}
    <div class="history-list">
      {#each items as item (item.id)}
        <article>
          <div class={`history-status ${item.state}`}>{item.state}</div>
          <div class="history-main">
            <h2>{basename(item.spec.source.primaryFile)}</h2>
            <p>{operationLabel(item.spec.operation)} · {date(item.finishedAt)}</p>
            {#if item.error}<p class="history-error">{item.error}</p>{/if}
            <dl>
              <div>
                <dt>Input</dt>
                <dd title={item.spec.source.primaryFile}>{item.spec.source.primaryFile}</dd>
              </div>
              <div>
                <dt>Output</dt>
                <dd title={item.spec.destination ?? ''}>{item.spec.destination ?? 'No output'}</dd>
              </div>
              <div>
                <dt>Size</dt>
                <dd>
                  {formatBytes(item.inputSize)} → {item.outputSize === null
                    ? '—'
                    : formatBytes(item.outputSize)}
                </dd>
              </div>
              <div>
                <dt>Started</dt>
                <dd>{date(item.startedAt)}</dd>
              </div>
            </dl>
            {#if item.chdInfo}
              <details class="chd-info">
                <summary>CHD information</summary>
                <dl>
                  <div>
                    <dt>Version</dt>
                    <dd>{item.chdInfo.formatVersion}</dd>
                  </div>
                  <div>
                    <dt>Media</dt>
                    <dd>{item.chdInfo.mediaKind.toUpperCase()}</dd>
                  </div>
                  <div>
                    <dt>Logical</dt>
                    <dd>{formatBytes(item.chdInfo.logicalSize)}</dd>
                  </div>
                  <div>
                    <dt>Compressed</dt>
                    <dd>{formatBytes(item.chdInfo.compressedSize)}</dd>
                  </div>
                  <div>
                    <dt>Codecs</dt>
                    <dd>{item.chdInfo.codecs.join(', ') || 'None'}</dd>
                  </div>
                  <div>
                    <dt>SHA-1</dt>
                    <dd class="hash">{item.chdInfo.hashes.sha1 ?? '—'}</dd>
                  </div>
                </dl>
              </details>
            {/if}
            <details>
              <summary>Process log ({item.log.length})</summary>
              <pre>{item.log.join('\n') || 'No process output was recorded.'}</pre>
            </details>
          </div>
          <div class="history-actions">
            {#if item.state !== 'completed'}<button type="button" onclick={() => onRetry(item.id)}
                >Retry</button
              >{/if}
            <button type="button" onclick={() => onRemove(item.id)}>Remove</button>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .history-view {
    padding: 30px clamp(22px, 5vw, 68px) 48px;
  }
  .history-heading {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding-bottom: 22px;
    border-bottom: 1px solid var(--alloy);
  }
  h1 {
    margin: 0;
    font: 600 clamp(31px, 4vw, 43px) / 1 var(--display);
  }
  .history-heading > span {
    color: var(--muted);
    font: 10px var(--mono);
    text-transform: uppercase;
  }
  .history-empty {
    display: grid;
    max-width: 460px;
    justify-items: start;
    margin: clamp(70px, 14vh, 140px) auto;
  }
  .history-empty > div {
    display: grid;
    width: 54px;
    height: 54px;
    place-items: center;
    margin-bottom: 20px;
    border: 1px solid var(--alloy);
    border-radius: 50%;
    color: var(--disc-blue);
    font: 22px var(--mono);
  }
  .history-empty h2 {
    margin: 0 0 10px;
    font: 600 clamp(28px, 4vw, 40px) / 1.05 var(--display);
  }
  .history-empty p {
    margin: 0 0 22px;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.6;
  }
  .history-list {
    display: grid;
    max-width: 1000px;
    margin: 24px auto;
    border-top: 1px solid var(--alloy);
  }
  article {
    display: grid;
    grid-template-columns: 90px 1fr auto;
    gap: 22px;
    padding: 22px 0;
    border-bottom: 1px solid var(--alloy);
  }
  .history-status {
    align-self: start;
    color: var(--oxide-teal);
    font: 600 9px var(--mono);
    text-transform: uppercase;
  }
  .history-status.failed,
  .history-status.interrupted,
  .history-status.blocked {
    color: var(--danger);
  }
  .history-main h2 {
    margin: 0;
    font: 600 21px var(--display);
  }
  .history-main > p {
    margin: 3px 0 14px;
    color: var(--muted);
    font-size: 10px;
  }
  .history-main > p.history-error {
    padding: 8px 10px;
    border-radius: 5px;
    color: var(--danger-text);
    background: color-mix(in srgb, var(--danger) 8%, var(--surface));
  }
  dl {
    display: grid;
    gap: 7px;
    margin: 0;
  }
  dl div {
    display: grid;
    grid-template-columns: 65px minmax(0, 1fr);
    gap: 10px;
  }
  dt {
    color: var(--muted);
    font: 9px var(--mono);
    text-transform: uppercase;
  }
  dd {
    min-width: 0;
    margin: 0;
    overflow: hidden;
    font-size: 10px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .hash {
    font-family: var(--mono);
  }
  details {
    margin-top: 12px;
  }
  .chd-info {
    padding: 10px;
    border: 1px solid var(--alloy);
    border-radius: 6px;
  }
  summary,
  .history-actions button {
    color: var(--disc-blue);
    font-size: 10px;
    font-weight: 600;
    cursor: pointer;
  }
  pre {
    overflow-x: auto;
    padding: 12px;
    border-radius: 6px;
    color: var(--basalt);
    background: var(--porcelain);
    font: 9px/1.55 var(--mono);
  }
  .history-actions {
    display: flex;
    gap: 6px;
    align-items: start;
  }
  .history-actions button {
    padding: 6px 8px;
    border: 1px solid var(--alloy);
    border-radius: 5px;
    background: transparent;
  }
  @media (max-width: 700px) {
    article {
      grid-template-columns: 1fr;
      gap: 10px;
    }
  }
</style>
