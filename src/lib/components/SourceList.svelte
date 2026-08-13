<script lang="ts">
  import { formatBytes, mediaLabel, t, type Locale } from '../i18n';
  import { basename } from '../presentation';
  import type { SourceSet } from '../types';
  import TrackBand from './TrackBand.svelte';

  export let sources: SourceSet[] = [];
  export let selected = 0;
  export let locale: Locale;
  export let inactive = false;
  export let onSelect: (index: number) => void;
  export let onRemove: (index: number) => void;
</script>

<section class="source-panel" aria-labelledby="sources-title" inert={inactive}>
  <div class="panel-heading">
    <div>
      <p class="section-label">{t(locale, 'importedSets')}</p>
      <h2 id="sources-title">{t(locale, 'sources')}</h2>
    </div>
    <span class="count">{sources.length.toString().padStart(2, '0')}</span>
  </div>
  <div class="source-list">
    {#each sources as source, index (source.primaryFile)}
      <div
        class:active={selected === index}
        class:invalid={source.validationProblems.length > 0}
        class="source-row"
      >
        <button
          type="button"
          class="source-select"
          onclick={() => onSelect(index)}
          aria-pressed={selected === index}
        >
          <span class="source-index">{(index + 1).toString().padStart(2, '0')}</span>
          <span class="source-summary">
            <span class="source-name">{basename(source.primaryFile)}</span>
            <span class="source-meta">
              {source.format.toUpperCase()} · {mediaLabel(locale, source.mediaKind)} · {formatBytes(
                locale,
                source.totalSize,
              )}
            </span>
            <TrackBand tracks={source.tracks} {locale} compact />
          </span>
          <span
            class:bad={source.validationProblems.length > 0}
            class="source-state"
            aria-hidden="true"
          ></span>
          <span class="sr-only">
            {source.validationProblems.length ? t(locale, 'needsAttention') : t(locale, 'ready')}
          </span>
        </button>
        <button
          class="remove"
          type="button"
          aria-label={t(locale, 'removeSource', { name: basename(source.primaryFile) })}
          onclick={() => onRemove(index)}>×</button
        >
      </div>
    {/each}
  </div>
</section>

<style>
  .source-panel {
    min-width: 0;
    border-right: 1px solid var(--alloy);
  }
  .panel-heading {
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
  .count {
    color: var(--muted);
    font: 11px var(--mono);
  }
  .source-list {
    display: grid;
  }
  .source-row {
    position: relative;
    border-bottom: 1px solid var(--alloy);
  }
  .source-row.active {
    background: color-mix(in srgb, var(--disc-blue) 8%, var(--surface));
    box-shadow: inset 3px 0 var(--disc-blue);
  }
  .source-row.invalid.active {
    box-shadow: inset 3px 0 var(--danger);
  }
  .source-select {
    display: grid;
    width: 100%;
    grid-template-columns: 28px minmax(0, 1fr) 8px;
    gap: 10px;
    align-items: start;
    padding: 16px 14px;
    border: 0;
    color: inherit;
    text-align: left;
    background: transparent;
    cursor: pointer;
  }
  .source-select:hover {
    background: color-mix(in srgb, var(--disc-blue) 6%, transparent);
  }
  .source-index {
    padding-top: 2px;
    color: var(--muted);
    font: 9px var(--mono);
  }
  .source-summary {
    display: grid;
    min-width: 0;
    gap: 5px;
  }
  .source-name {
    overflow: hidden;
    font-size: 12px;
    font-weight: 650;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .source-meta {
    overflow: hidden;
    color: var(--muted);
    font: 9px var(--mono);
    letter-spacing: 0.025em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .source-state {
    width: 7px;
    height: 7px;
    margin-top: 3px;
    border-radius: 50%;
    background: var(--oxide-teal);
  }
  .source-state.bad {
    background: var(--danger);
  }
  .remove {
    position: absolute;
    top: 5px;
    right: 5px;
    display: grid;
    width: 28px;
    height: 28px;
    place-items: center;
    border: 0;
    border-radius: 50%;
    color: var(--muted);
    background: transparent;
    opacity: 0;
    cursor: pointer;
  }
  .source-row:hover .remove,
  .remove:focus-visible {
    opacity: 1;
  }
</style>
