export type MediaKind = 'cd' | 'dvd' | 'unknownOptical';
export type SourceFormat = 'cue' | 'gdi' | 'iso' | 'chd';
export type TrackKind = 'data' | 'audio' | 'subchannel' | 'unknown';
export type Operation = 'createCd' | 'createDvd' | 'extractCd' | 'extractDvd' | 'verify' | 'info';

export interface Track {
  number: number;
  kind: TrackKind;
  sourceFile: string;
  startLba: number | null;
  sectorSize: number | null;
}

export interface ValidationProblem {
  kind:
    | 'missingReference'
    | 'duplicateReference'
    | 'escapingReference'
    | 'unreadableReference'
    | 'unreadablePrimary'
    | 'malformedDescriptor'
    | 'duplicateTrack'
    | 'trackCountMismatch';
  line: number | null;
  reference: string | null;
}

export interface SourceSet {
  primaryFile: string;
  referencedFiles: string[];
  format: SourceFormat;
  mediaKind: MediaKind;
  tracks: Track[];
  totalSize: number;
  validationProblems: ValidationProblem[];
}

export interface DiscoveryIssue {
  kind: 'inputNotFound' | 'inputUnreadable' | 'unsupportedInput';
  path: string;
}

export interface DiscoveryReport {
  sourceSets: SourceSet[];
  issues: DiscoveryIssue[];
}

export interface AdvancedOptions {
  splitBin: boolean;
  processors: number | null;
  hunkSize: number | null;
}

export interface JobOptions {
  splitBin: boolean;
  processors: number | null;
  hunkSize: number | null;
}

export interface JobSpec {
  source: SourceSet;
  operation: Operation;
  destination: string | null;
  options: JobOptions;
}

export type JobState =
  | 'queued'
  | 'preflight'
  | 'running'
  | 'verifying'
  | 'completed'
  | 'failed'
  | 'cancelled'
  | 'interrupted'
  | 'blocked';

export interface JobProgress {
  phase: 'inspecting' | 'compressing' | 'extracting' | 'verifying' | 'complete' | 'unknown';
  percentage: number | null;
  processedBytes: number | null;
  elapsedMillis: number | null;
  message: string;
}

export interface ChdInfo {
  formatVersion: number;
  mediaKind: MediaKind;
  codecs: string[];
  logicalSize: number;
  compressedSize: number;
  ratio: number | null;
  hunkSize: number;
  totalHunks: number;
  unitSize: number;
  totalUnits: number;
  hashes: { sha1: string | null; dataSha1: string | null; parentSha1: string | null };
  tracks: Array<{
    number: number;
    kind: TrackKind;
    frames: number | null;
    pregap: number | null;
    postgap: number | null;
  }>;
  metadata: Array<{ tag: string; index: number; length: number; value: string }>;
}

export interface JobRecord {
  id: string;
  spec: JobSpec;
  state: JobState;
  progress: JobProgress | null;
  message: string;
  error: string | null;
  createdAt: number;
  startedAt: number | null;
  finishedAt: number | null;
  inputSize: number;
  outputSize: number | null;
  log: string[];
  chdInfo: ChdInfo | null;
  temporaryPaths: string[];
}

export interface QueueSnapshot {
  paused: boolean;
  activeJobId: string | null;
  jobs: JobRecord[];
}

export interface Settings {
  destinationDirectory: string | null;
  locale: import('./i18n').Locale | null;
}
