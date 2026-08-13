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

export interface QueueItem {
  id: string;
  source: SourceSet;
  operation: Operation;
  destination: string | null;
  status: 'queued' | 'blocked';
  message: string;
  createdAt: Date;
}

export interface HistoryItem {
  id: string;
  sourceName: string;
  sourcePath: string;
  destination: string | null;
  operation: Operation;
  status: 'completed' | 'failed' | 'cancelled' | 'interrupted';
  startedAt: Date;
  finishedAt: Date;
  inputSize: number;
  outputSize: number | null;
  log: string[];
}
