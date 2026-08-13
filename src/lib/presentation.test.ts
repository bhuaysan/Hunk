import { describe, expect, it } from 'vitest';
import { defaultDestination, formatBytes, operationsFor, replaceExtension } from './presentation';
import type { SourceSet } from './types';

const source: SourceSet = {
  primaryFile: '/games/Disc [日本].iso',
  referencedFiles: [],
  format: 'iso',
  mediaKind: 'unknownOptical',
  tracks: [],
  totalSize: 4_700_000_000,
  validationProblems: [],
};

describe('workbench presentation', () => {
  it('requires an explicit ISO media choice', () => {
    expect(operationsFor(source)).toEqual([]);
    expect(operationsFor(source, 'cd')).toEqual(['createCd']);
    expect(operationsFor(source, 'dvd')).toEqual(['createDvd']);
  });

  it('does not offer operations for invalid sources', () => {
    const invalid = {
      ...source,
      validationProblems: [{ kind: 'missingReference' as const, line: 2, reference: 'track.bin' }],
    };
    expect(operationsFor(invalid, 'dvd')).toEqual([]);
  });

  it('derives safe adjacent output names without changing the input path', () => {
    expect(replaceExtension(source.primaryFile, '.chd')).toBe('/games/Disc [日本].chd');
    expect(defaultDestination(source, 'createDvd')).toBe('/games/Disc [日本].chd');
    expect(defaultDestination(source, 'verify')).toBeNull();
  });

  it('keeps the CUE descriptor name stable for split-track extraction', () => {
    const chd = { ...source, primaryFile: '/games/Disc.chd', format: 'chd' as const };
    expect(defaultDestination(chd, 'extractCd', true)).toBe('/games/Disc.cue');
  });

  it('formats source sizes for compact list rows', () => {
    expect(formatBytes(999)).toBe('999 B');
    expect(formatBytes(1_250_000)).toBe('1.25 MB');
  });
});
