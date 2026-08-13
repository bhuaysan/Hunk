<script lang="ts">
  import { t, type Locale } from '../i18n';

  export let importing = false;
  export let hovering = false;
  export let locale: Locale;
  export let onFiles: () => void;
  export let onFolder: () => void;
</script>

<section class:hovering class="import-surface" aria-labelledby="import-title" aria-busy={importing}>
  <div class="disc-glyph" aria-hidden="true"><span></span></div>
  <div class="import-copy">
    <p class="section-label">{t(locale, 'bringInMedia')}</p>
    <h2 id="import-title">{t(locale, 'dropDiscSet')}</h2>
    <p>{t(locale, 'importExplanation')}</p>
  </div>
  <div class="import-actions">
    <button class="primary" type="button" onclick={onFiles} disabled={importing}>
      {importing ? t(locale, 'inspecting') : t(locale, 'chooseImages')}
    </button>
    <button class="secondary" type="button" onclick={onFolder} disabled={importing}
      >{t(locale, 'chooseFolder')}</button
    >
  </div>
  {#if hovering}<div class="drop-message" role="status">{t(locale, 'releaseToInspect')}</div>{/if}
</section>

<style>
  .import-surface {
    position: relative;
    display: grid;
    grid-template-columns: auto minmax(240px, 1fr) auto;
    gap: 22px;
    align-items: center;
    min-height: 132px;
    padding: 24px 26px;
    overflow: hidden;
    border: 1px dashed color-mix(in srgb, var(--disc-blue) 48%, var(--alloy));
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--surface) 76%, transparent);
  }
  .import-surface.hovering {
    border-color: var(--oxide-teal);
    background: color-mix(in srgb, var(--oxide-teal) 8%, var(--surface));
  }
  .disc-glyph {
    position: relative;
    width: 76px;
    aspect-ratio: 1;
    border: 1px solid color-mix(in srgb, var(--disc-blue) 38%, transparent);
    border-radius: 50%;
    background: repeating-radial-gradient(
      circle,
      color-mix(in srgb, var(--disc-blue) 15%, transparent) 0 1px,
      transparent 1px 7px
    );
  }
  .disc-glyph::before,
  .disc-glyph span {
    position: absolute;
    border-radius: 50%;
    content: '';
  }
  .disc-glyph::before {
    inset: 31%;
    border: 1px solid color-mix(in srgb, var(--basalt) 18%, transparent);
  }
  .disc-glyph span {
    inset: 43%;
    background: var(--surface);
    box-shadow: 0 0 0 1px var(--alloy);
  }
  h2 {
    margin: 0 0 5px;
    font: 600 clamp(25px, 3vw, 34px) / 1 var(--display);
    letter-spacing: -0.025em;
  }
  .import-copy > p:last-child {
    max-width: 570px;
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.55;
  }
  .import-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 8px;
  }
  .drop-message {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: var(--basalt);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    font: 600 18px var(--display);
    letter-spacing: 0.01em;
  }
  @media (max-width: 850px) {
    .import-surface {
      grid-template-columns: auto 1fr;
    }
    .import-actions {
      grid-column: 2;
      justify-content: flex-start;
    }
  }
  @media (max-width: 560px) {
    .import-surface {
      grid-template-columns: 1fr;
      padding: 20px;
    }
    .disc-glyph {
      display: none;
    }
    .import-actions {
      grid-column: auto;
    }
  }
</style>
