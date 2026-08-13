import { describe, expect, it } from 'vitest';
import {
  detectLocale,
  dictionaries,
  formatBytes,
  formatDate,
  formatNumber,
  operationLabel,
  problemMessage,
  t,
} from './i18n';

describe('localization', () => {
  it('keeps the English and German dictionaries complete and aligned', () => {
    expect(Object.keys(dictionaries.de).sort()).toEqual(Object.keys(dictionaries.en).sort());
    expect(Object.values(dictionaries.en).every(Boolean)).toBe(true);
    expect(Object.values(dictionaries.de).every(Boolean)).toBe(true);
  });

  it('selects supported locales from system language tags', () => {
    expect(detectLocale('de-DE')).toBe('de');
    expect(detectLocale('de-CH')).toBe('de');
    expect(detectLocale('en-GB')).toBe('en');
    expect(detectLocale('fr-FR')).toBe('en');
  });

  it('interpolates translated interface copy', () => {
    expect(t('en', 'addedToQueue', { name: 'Disc.chd' })).toBe('Disc.chd was added to the queue.');
    expect(t('de', 'addedToQueue', { name: 'Disc.chd' })).toBe(
      'Disc.chd wurde in die Warteschlange eingereiht.',
    );
  });

  it('localizes operations and structured validation problems', () => {
    expect(operationLabel('en', 'createCd')).toBe('Create CD image');
    expect(operationLabel('de', 'createCd')).toBe('CD-Image erstellen');
    const problem = { kind: 'missingReference' as const, line: 2, reference: 'track.bin' };
    expect(problemMessage('en', problem)).toContain('Line 2.');
    expect(problemMessage('de', problem)).toContain('Zeile 2.');
  });

  it('formats numbers, sizes, and dates with the selected locale', () => {
    expect(formatNumber('en', 12_345.6)).toBe('12,345.6');
    expect(formatNumber('de', 12_345.6)).toBe('12.345,6');
    expect(formatBytes('en', 1_250_000)).toBe('1.25 MB');
    expect(formatBytes('de', 1_250_000)).toBe('1,25 MB');

    const timestamp = new Date(2024, 0, 2, 16, 4).getTime();
    expect(formatDate('en', timestamp)).not.toBe(formatDate('de', timestamp));
  });
});
