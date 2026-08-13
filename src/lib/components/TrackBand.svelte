<script lang="ts">
  import { formatNumber, t, trackLabel, type Locale } from '../i18n';
  import type { Track } from '../types';

  export let tracks: Track[] = [];
  export let progress: number | null = null;
  export let compact = false;
  export let locale: Locale;

  $: visibleTracks = tracks.length
    ? tracks
    : [{ number: 1, kind: 'unknown' as const, sourceFile: '', startLba: null, sectorSize: null }];
  $: summary = [
    ...visibleTracks.map((track) =>
      t(locale, 'trackDescription', {
        number: formatNumber(locale, track.number),
        kind: trackLabel(locale, track.kind),
      }),
    ),
    ...(progress === null
      ? []
      : [t(locale, 'progressComplete', { value: formatNumber(locale, Math.round(progress)) })]),
  ].join(', ');
</script>

<div class:compact class="track-figure" role="img" aria-label={summary}>
  {#if !compact}
    <div class="track-labels" aria-hidden="true">
      {#each visibleTracks as track}
        <span>{track.number.toString().padStart(2, '0')} · {trackLabel(locale, track.kind)}</span>
      {/each}
    </div>
  {/if}
  <div class="track-band" style={`--segments:${visibleTracks.length}`} aria-hidden="true">
    {#each visibleTracks as track}
      <span class:unknown={track.kind === 'unknown'} class={track.kind}></span>
    {/each}
    {#if progress !== null}
      <i style={`width:${Math.max(0, Math.min(100, progress))}%`}></i>
    {/if}
  </div>
</div>

<style>
  .track-figure {
    min-width: 0;
  }
  .track-labels,
  .track-band {
    display: grid;
    grid-template-columns: repeat(var(--segments), minmax(16px, 1fr));
  }
  .track-labels {
    gap: 4px;
    margin-bottom: 7px;
    color: var(--muted);
    font: 600 9px var(--mono);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .track-labels span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .track-band {
    position: relative;
    height: 14px;
    gap: 3px;
    overflow: hidden;
    border-radius: 3px;
  }
  .track-band span {
    min-width: 0;
    background: repeating-linear-gradient(135deg, var(--disc-blue) 0 5px, #6092b9 5px 9px);
  }
  .track-band .audio {
    background: repeating-linear-gradient(90deg, var(--audio-amber) 0 3px, #e4b565 3px 6px);
  }
  .track-band .subchannel {
    background:
      repeating-linear-gradient(135deg, var(--oxide-teal) 0 2px, transparent 2px 5px),
      color-mix(in srgb, var(--oxide-teal) 25%, var(--surface));
  }
  .track-band .unknown {
    border: 1px dashed var(--muted);
    background: transparent;
  }
  .track-band i {
    position: absolute;
    inset: 0 auto 0 0;
    border-right: 2px solid var(--surface);
    background: rgb(255 255 255 / 28%);
    box-shadow: 8px 0 18px rgb(23 36 43 / 24%);
    pointer-events: none;
  }
  .compact .track-band {
    height: 7px;
    gap: 2px;
    opacity: 0.82;
  }
</style>
