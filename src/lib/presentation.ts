import type { MediaKind, Operation, SourceSet, ValidationProblem } from './types';

const operationLabels: Record<Operation, string> = {
  createCd: 'Create CD image',
  createDvd: 'Create DVD image',
  extractCd: 'Extract CD image',
  extractDvd: 'Extract DVD image',
  verify: 'Verify integrity',
  info: 'Read information',
};

const problemMessages: Record<ValidationProblem['kind'], string> = {
  missingReference: 'A referenced track file is missing.',
  duplicateReference: 'A track file is referenced more than once.',
  escapingReference: 'A track reference points outside the descriptor folder.',
  unreadableReference: 'A referenced track file cannot be read.',
  unreadablePrimary: 'The source file cannot be read.',
  malformedDescriptor: 'The track descriptor is malformed.',
  duplicateTrack: 'A track number is used more than once.',
  trackCountMismatch: 'The declared and discovered track counts differ.',
};

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
      return replaceExtension(source.primaryFile, splitBin ? ' - Track %t.cue' : '.cue');
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

export function operationLabel(operation: Operation): string {
  return operationLabels[operation];
}

export function problemMessage(problem: ValidationProblem): string {
  const location = problem.line ? ` Line ${problem.line}.` : '';
  const reference = problem.reference ? ` ${problem.reference}` : '';
  return `${problemMessages[problem.kind]}${reference}${location}`;
}

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  const digits = value >= 100 || unit === 0 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${units[unit]}`;
}

export function formatMedia(kind: MediaKind): string {
  if (kind === 'unknownOptical') return 'Choose CD or DVD';
  return kind.toUpperCase();
}
