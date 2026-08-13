<script lang="ts">
  import {
    basename,
    dirname,
    formatBytes,
    formatMedia,
    operationLabel,
    problemMessage,
  } from '../presentation';
  import type { AdvancedOptions, MediaKind, Operation, SourceSet } from '../types';
  import TrackBand from './TrackBand.svelte';

  export let source: SourceSet;
  export let mediaChoice: MediaKind | undefined;
  export let operations: Operation[] = [];
  export let operation: Operation | undefined;
  export let destination: string | null = null;
  export let advanced: AdvancedOptions;
  export let onMediaChoice: (kind: MediaKind) => void;
  export let onOperation: (operation: Operation) => void;
  export let onDestination: (value: string) => void;
  export let onChooseDestination: () => void;
  export let onQueue: () => void;

  $: needsDestination = operation?.startsWith('create') || operation?.startsWith('extract');
  $: canQueue =
    !!operation &&
    source.validationProblems.length === 0 &&
    (!needsDestination || Boolean(destination?.trim()));
</script>

<section class="inspector" aria-labelledby="inspector-title">
  <header class="inspector-heading">
    <div>
      <p class="section-label">Selected source</p>
      <h2 id="inspector-title">{basename(source.primaryFile)}</h2>
    </div>
    <span class:invalid={source.validationProblems.length > 0} class="ready-pill">
      {source.validationProblems.length ? 'Needs attention' : 'Ready'}
    </span>
  </header>

  <div class="disc-map">
    <div class="disc-summary">
      <span>{source.format.toUpperCase()}</span>
      <strong>{source.tracks.length || '—'}</strong>
      <small>{source.tracks.length === 1 ? 'track' : 'tracks'}</small>
    </div>
    <TrackBand tracks={source.tracks} />
    <div class="legend" aria-label="Track legend">
      <span class="data">Data</span><span class="audio">Audio</span><span class="subchannel"
        >Subchannel</span
      >
    </div>
  </div>

  {#if source.validationProblems.length > 0}
    <div class="problem-box" role="alert">
      <strong>This source cannot be queued.</strong>
      {#each source.validationProblems as problem}
        <p>{problemMessage(problem)}</p>
      {/each}
    </div>
  {:else}
    <div class="facts">
      <div><span>Media</span><strong>{formatMedia(mediaChoice ?? source.mediaKind)}</strong></div>
      <div><span>Source size</span><strong>{formatBytes(source.totalSize)}</strong></div>
      <div><span>Dependencies</span><strong>{source.referencedFiles.length}</strong></div>
      <div>
        <span>Location</span><strong title={dirname(source.primaryFile)}
          >{dirname(source.primaryFile) || '—'}</strong
        >
      </div>
    </div>

    {#if source.format === 'iso'}
      <fieldset class="media-choice">
        <legend>What kind of disc is this ISO?</legend>
        <p>ISO size is not a reliable media detector. Choose explicitly.</p>
        <div class="segment-control">
          <button
            type="button"
            class:active={mediaChoice === 'cd'}
            aria-pressed={mediaChoice === 'cd'}
            onclick={() => onMediaChoice('cd')}>CD</button
          >
          <button
            type="button"
            class:active={mediaChoice === 'dvd'}
            aria-pressed={mediaChoice === 'dvd'}
            onclick={() => onMediaChoice('dvd')}>DVD</button
          >
        </div>
      </fieldset>
    {/if}

    {#if operations.length > 0}
      <fieldset class="operations">
        <legend>Choose an action</legend>
        <div class="operation-grid">
          {#each operations as item}
            <button
              type="button"
              class:active={operation === item}
              aria-pressed={operation === item}
              onclick={() => onOperation(item)}
            >
              <span aria-hidden="true"
                >{item.startsWith('create')
                  ? '＋'
                  : item.startsWith('extract')
                    ? '↙'
                    : item === 'verify'
                      ? '✓'
                      : 'i'}</span
              >
              {operationLabel(item)}
            </button>
          {/each}
        </div>
      </fieldset>
    {:else if source.format === 'iso'}
      <p class="choice-prompt">Choose CD or DVD to continue.</p>
    {/if}

    {#if operation}
      {#if needsDestination}
        <div class="destination-field">
          <label for="destination">Destination</label>
          <div>
            <input
              id="destination"
              value={destination ?? ''}
              oninput={(event) => onDestination(event.currentTarget.value)}
              spellcheck="false"
            />
            <button
              type="button"
              class="browse"
              onclick={onChooseDestination}
              aria-label="Choose destination folder">…</button
            >
          </div>
          <p>Existing files are never overwritten.</p>
        </div>
      {/if}

      {#if operation.startsWith('create') || operation === 'extractCd'}
        <details class="advanced-options">
          <summary>Advanced options</summary>
          <div class="advanced-grid">
            {#if operation === 'extractCd'}
              <label class="check-row">
                <input type="checkbox" bind:checked={advanced.splitBin} />
                <span>Split BIN per track</span>
              </label>
            {:else}
              <label>
                <span>Processors</span>
                <input
                  type="number"
                  min="1"
                  max="256"
                  placeholder="Automatic"
                  bind:value={advanced.processors}
                />
              </label>
              <label>
                <span>Hunk size (bytes)</span>
                <input
                  type="number"
                  min="1"
                  max="1048576"
                  placeholder="chdman default"
                  bind:value={advanced.hunkSize}
                />
              </label>
            {/if}
          </div>
        </details>
      {/if}

      <div class="action-footer">
        <p>
          {operation === 'verify' || operation === 'info'
            ? 'This action reads the CHD without changing it.'
            : 'The source stays untouched. New CHDs are verified before publication.'}
        </p>
        <button class="primary queue-button" type="button" disabled={!canQueue} onclick={onQueue}>
          {operationLabel(operation)}
        </button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .inspector {
    min-width: 0;
    padding: 0 24px 24px;
  }
  .inspector-heading {
    display: flex;
    gap: 16px;
    align-items: flex-end;
    justify-content: space-between;
    min-height: 72px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--alloy);
  }
  h2 {
    max-width: 540px;
    margin: 0;
    overflow: hidden;
    font: 600 clamp(23px, 3vw, 31px) / 1 var(--display);
    letter-spacing: -0.02em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ready-pill {
    flex: none;
    padding: 5px 8px;
    border: 1px solid color-mix(in srgb, var(--oxide-teal) 40%, transparent);
    border-radius: 999px;
    color: var(--oxide-teal);
    font: 600 9px var(--mono);
    text-transform: uppercase;
  }
  .ready-pill.invalid {
    border-color: color-mix(in srgb, var(--danger) 40%, transparent);
    color: var(--danger);
  }
  .disc-map {
    display: grid;
    grid-template-columns: 74px minmax(0, 1fr);
    gap: 14px 18px;
    align-items: end;
    padding: 24px 0 18px;
  }
  .disc-summary {
    display: grid;
    grid-row: 1 / span 2;
    align-content: center;
    min-height: 74px;
    padding: 8px;
    border: 1px solid var(--alloy);
    border-radius: 50%;
    text-align: center;
  }
  .disc-summary span,
  .disc-summary small {
    color: var(--muted);
    font: 8px var(--mono);
    text-transform: uppercase;
  }
  .disc-summary strong {
    font: 600 21px/1 var(--display);
  }
  .legend {
    display: flex;
    gap: 14px;
    color: var(--muted);
    font: 9px var(--mono);
  }
  .legend span::before {
    display: inline-block;
    width: 7px;
    height: 7px;
    margin-right: 5px;
    border-radius: 1px;
    background: var(--disc-blue);
    content: '';
  }
  .legend .audio::before {
    background: var(--audio-amber);
  }
  .legend .subchannel::before {
    border: 1px dashed var(--oxide-teal);
    background: transparent;
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    border-block: 1px solid var(--alloy);
  }
  .facts div {
    display: grid;
    gap: 4px;
    min-width: 0;
    padding: 13px 12px;
    border-right: 1px solid var(--alloy);
  }
  .facts div:last-child {
    grid-column: 1 / -1;
    border-top: 1px solid var(--alloy);
    border-right: 0;
  }
  .facts span,
  .destination-field > label,
  .advanced-grid label > span {
    color: var(--muted);
    font: 600 9px var(--mono);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .facts strong {
    min-width: 0;
    overflow: hidden;
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  fieldset {
    min-width: 0;
    margin: 20px 0 0;
    padding: 0;
    border: 0;
  }
  legend {
    margin-bottom: 9px;
    font: 600 13px var(--body);
  }
  .media-choice > p,
  .choice-prompt {
    margin: -4px 0 11px;
    color: var(--muted);
    font-size: 11px;
  }
  .segment-control {
    display: grid;
    grid-template-columns: 1fr 1fr;
    padding: 3px;
    border: 1px solid var(--alloy);
    border-radius: 7px;
    background: var(--porcelain);
  }
  .segment-control button {
    min-height: 34px;
    border: 0;
    border-radius: 5px;
    color: var(--muted);
    background: transparent;
    cursor: pointer;
  }
  .segment-control button.active {
    color: var(--basalt);
    background: var(--surface);
    box-shadow: 0 1px 4px rgb(23 36 43 / 10%);
  }
  .operation-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(135px, 1fr));
    gap: 7px;
  }
  .operation-grid button {
    display: flex;
    gap: 8px;
    align-items: center;
    min-height: 42px;
    padding: 0 11px;
    border: 1px solid var(--alloy);
    border-radius: 7px;
    color: var(--basalt);
    font-size: 11px;
    font-weight: 600;
    text-align: left;
    background: transparent;
    cursor: pointer;
  }
  .operation-grid button span {
    display: grid;
    width: 20px;
    height: 20px;
    flex: none;
    place-items: center;
    border-radius: 50%;
    color: var(--disc-blue);
    background: color-mix(in srgb, var(--disc-blue) 10%, transparent);
    font: 12px var(--mono);
  }
  .operation-grid button.active {
    border-color: var(--disc-blue);
    background: color-mix(in srgb, var(--disc-blue) 7%, transparent);
  }
  .destination-field {
    margin-top: 18px;
  }
  .destination-field > div {
    display: grid;
    grid-template-columns: 1fr 38px;
    margin-top: 6px;
  }
  .destination-field input {
    min-width: 0;
    border-radius: 6px 0 0 6px;
  }
  .browse {
    border: 1px solid var(--alloy);
    border-left: 0;
    border-radius: 0 6px 6px 0;
    color: var(--basalt);
    background: var(--surface);
    cursor: pointer;
  }
  .destination-field p {
    margin: 5px 0 0;
    color: var(--muted);
    font-size: 10px;
  }
  .advanced-options {
    margin-top: 14px;
    border-top: 1px solid var(--alloy);
  }
  .advanced-options summary {
    padding: 12px 0;
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }
  .advanced-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    padding-bottom: 10px;
  }
  .advanced-grid label {
    display: grid;
    gap: 5px;
  }
  .advanced-grid .check-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .check-row input {
    width: 16px;
    height: 16px;
  }
  .action-footer {
    display: flex;
    gap: 18px;
    align-items: center;
    justify-content: space-between;
    margin-top: 18px;
    padding-top: 16px;
    border-top: 1px solid var(--alloy);
  }
  .action-footer p {
    max-width: 360px;
    margin: 0;
    color: var(--muted);
    font-size: 10px;
    line-height: 1.5;
  }
  .queue-button {
    flex: none;
  }
  .problem-box {
    margin-top: 22px;
    padding: 14px 16px;
    border: 1px solid color-mix(in srgb, var(--danger) 30%, var(--alloy));
    border-radius: 7px;
    color: var(--danger-text);
    background: color-mix(in srgb, var(--danger) 7%, var(--surface));
  }
  .problem-box strong {
    font-size: 12px;
  }
  .problem-box p {
    margin: 7px 0 0;
    font-size: 11px;
    line-height: 1.45;
  }
  @media (max-width: 700px) {
    .inspector {
      padding-inline: 18px;
    }
    .action-footer {
      align-items: stretch;
      flex-direction: column;
    }
  }
</style>
