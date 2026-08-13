import type { MediaKind, Operation, SourceSet } from './types';

export function basename(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

export function dirname(path: string): string {
  const separator = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return separator < 0 ? '' : path.slice(0, separator);
}

export function replaceExtension(path: string, extension: string): string {
  const separator = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  const dot = path.lastIndexOf('.');
  const stem = dot > separator ? path.slice(0, dot) : path;
  return `${stem}${extension}`;
}

export function defaultDestination(
  source: SourceSet,
  operation: Operation,
  splitBin = false,
): string | null {
  switch (operation) {
    case 'createCd':
    case 'createDvd':
      return replaceExtension(source.primaryFile, '.chd');
    case 'extractCd':
      return replaceExtension(source.primaryFile, '.cue');
    case 'extractDvd':
      return replaceExtension(source.primaryFile, '.iso');
    case 'verify':
    case 'info':
      return null;
  }
}

export function operationsFor(source: SourceSet, isoKind?: MediaKind): Operation[] {
  if (source.validationProblems.length > 0) return [];
  if (source.format === 'chd') {
    if (source.mediaKind === 'cd') return ['extractCd', 'verify', 'info'];
    if (source.mediaKind === 'dvd') return ['extractDvd', 'verify', 'info'];
    return ['verify', 'info'];
  }
  const mediaKind = source.format === 'iso' ? isoKind : source.mediaKind;
  if (mediaKind === 'cd') return ['createCd'];
  if (mediaKind === 'dvd') return ['createDvd'];
  return [];
}
