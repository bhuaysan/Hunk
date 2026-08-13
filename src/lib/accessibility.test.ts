import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const css = readFileSync(new URL('../app.css', import.meta.url), 'utf8');
const app = readFileSync(new URL('../App.svelte', import.meta.url), 'utf8');

function luminance(hex: string): number {
  const channels = hex
    .slice(1)
    .match(/../g)!
    .map((channel) => Number.parseInt(channel, 16) / 255)
    .map((channel) => (channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4));
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(foreground: string, background: string): number {
  const values = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return (values[0] + 0.05) / (values[1] + 0.05);
}

describe('accessibility safeguards', () => {
  it('keeps small interface text at WCAG AA contrast in both themes', () => {
    const lightBackground = '#f4f7f6';
    const darkBackground = '#17242b';

    for (const color of ['#356f9c', '#247268', '#8a5b18', '#843b36', '#62747b']) {
      expect(contrast(color, lightBackground)).toBeGreaterThanOrEqual(4.5);
    }
    for (const color of ['#79b2df', '#67b5a9', '#f1bd68', '#f1aaa4', '#9aabad']) {
      expect(contrast(color, darkBackground)).toBeGreaterThanOrEqual(4.5);
    }
    expect(contrast('#ffffff', '#b9564e')).toBeGreaterThanOrEqual(4.5);
    expect(contrast('#ffffff', '#9d4842')).toBeGreaterThanOrEqual(4.5);
    for (const token of [
      '--interactive: #356f9c',
      '--interactive: #79b2df',
      '--success-text: #247268',
      '--success-text: #67b5a9',
      '--danger-fill: #9d4842',
    ]) {
      expect(css).toContain(token);
    }
  });

  it('keeps focus and reduced-motion affordances in the global styles', () => {
    expect(css).toContain(':focus-visible');
    expect(css).toContain('@media (prefers-reduced-motion: reduce)');
    expect(css).toContain('transition-duration: 0.01ms !important');
  });

  it('keeps overlays out of the focus order and keyboard dismissible', () => {
    expect(app).toContain('inert={narrowInspector && !inspectorOpen}');
    expect(app).toContain('inert={narrowInspector && inspectorOpen}');
    expect(app).toContain("event.key === 'Escape'");
    expect(app).toContain('node.showModal()');
    expect(app).toContain('oncancel=');
  });
});
