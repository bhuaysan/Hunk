import type { JobState, MediaKind, Operation, TrackKind, ValidationProblem } from './types';

export type Locale = 'en' | 'de';
type Parameters = Record<string, string | number>;

const en = {
  appTitle: 'Hunk — Optical workbench',
  primaryNavigation: 'Primary navigation',
  workspaceNavigation: 'Workspace',
  workbench: 'Workbench',
  history: 'History',
  localOnly: 'Local only',
  language: 'Language',
  english: 'English',
  german: 'Deutsch',
  localOpticalTools: 'Local optical media tools',
  ready: 'Ready',
  needsAttention: 'Needs attention',
  sourcesSafety: 'Sources stay untouched. Hunk never uploads media or overwrites output.',
  images: 'Images',
  folder: 'Folder',
  sourceDetails: 'Source details',
  closeDetails: 'Close source details',
  dismissMessage: 'Dismiss message',
  activeJob: 'Active job',
  closeQuestion: 'Stop processing and close Hunk?',
  closeExplanation:
    'The active chdman process will be cancelled. Source files and existing outputs stay untouched.',
  keepWorking: 'Keep working',
  cancelAndClose: 'Cancel job and close',
  importedSets: 'Imported sets',
  sources: 'Sources',
  removeSource: 'Remove {name}',
  bringInMedia: 'Bring in optical media',
  dropDiscSet: 'Drop a disc set here.',
  importExplanation:
    'CUE, GDI, ISO, and CHD stay on this computer. Referenced tracks are grouped automatically.',
  inspecting: 'Inspecting…',
  chooseImages: 'Choose images',
  chooseFolder: 'Choose folder',
  releaseToInspect: 'Release to inspect',
  selectedSource: 'Selected source',
  track: 'track',
  tracks: 'tracks',
  trackDescription: 'Track {number}: {kind}',
  progressComplete: '{value}% complete',
  trackLegend: 'Track legend',
  data: 'Data',
  audio: 'Audio',
  subchannel: 'Subchannel',
  unknown: 'Unknown',
  cannotQueue: 'This source cannot be queued.',
  media: 'Media',
  sourceSize: 'Source size',
  dependencies: 'Dependencies',
  location: 'Location',
  isoQuestion: 'What kind of disc is this ISO?',
  isoExplanation: 'ISO size is not a reliable media detector. Choose explicitly.',
  chooseAction: 'Choose an action',
  chooseMedia: 'Choose CD or DVD',
  chooseMediaToContinue: 'Choose CD or DVD to continue.',
  destination: 'Destination',
  chooseDestination: 'Choose destination folder',
  neverOverwrite: 'Existing files are never overwritten.',
  advancedOptions: 'Advanced options',
  splitBin: 'Split BIN per track',
  processors: 'Processors',
  automatic: 'Automatic',
  hunkSize: 'Hunk size (bytes)',
  chdmanDefault: 'chdman default',
  readOnlyAction: 'This action reads the CHD without changing it.',
  safeCreateAction: 'The source stays untouched. New CHDs are verified before publication.',
  createCd: 'Create CD image',
  createDvd: 'Create DVD image',
  extractCd: 'Extract CD image',
  extractDvd: 'Extract DVD image',
  verify: 'Verify integrity',
  info: 'Read information',
  missingReference: 'A referenced track file is missing.',
  duplicateReference: 'A track file is referenced more than once.',
  escapingReference: 'A track reference points outside the descriptor folder.',
  unreadableReference: 'A referenced track file cannot be read.',
  unreadablePrimary: 'The source file cannot be read.',
  malformedDescriptor: 'The track descriptor is malformed.',
  duplicateTrack: 'A track number is used more than once.',
  trackCountMismatch: 'The declared and discovered track counts differ.',
  line: 'Line {number}.',
  serialProcessing: 'Serial processing',
  queue: 'Queue',
  resume: 'Resume',
  pause: 'Pause',
  noQueuedJobs: 'No queued jobs',
  queueEmptyExplanation: 'Choose an action for a source. Jobs run safely, one at a time.',
  queuePaused: 'Queued work is paused. The active job may finish.',
  retryJob: 'Retry job',
  cancelJob: 'Cancel job',
  removeJob: 'Remove job',
  queued: 'Queued',
  preflight: 'Preflight',
  running: 'Running',
  verifying: 'Verifying',
  completed: 'Completed',
  failed: 'Failed',
  cancelled: 'Cancelled',
  interrupted: 'Interrupted',
  blocked: 'Blocked',
  waitingInQueue: 'Waiting in the serial queue',
  interruptedPreviously: 'Interrupted when Hunk last stopped',
  cancelledBeforeProcessing: 'Cancelled before processing',
  checkingSource: 'Checking source and destination',
  fullyVerifying: 'Fully verifying the temporary CHD',
  preflightBlocked: 'Preflight blocked this job',
  chdmanFailed: 'chdman could not complete the job',
  latestJobs: 'Latest 100 jobs',
  record: 'record',
  records: 'records',
  historyEmpty: 'Completed work will collect here.',
  historyExplanation:
    'Each record includes paths, size savings, timestamps, status, and a bounded process log.',
  openWorkbench: 'Open workbench',
  input: 'Input',
  output: 'Output',
  noOutput: 'No output',
  size: 'Size',
  started: 'Started',
  chdInformation: 'CHD information',
  version: 'Version',
  logical: 'Logical',
  compressed: 'Compressed',
  codecs: 'Codecs',
  none: 'None',
  processLog: 'Process log ({count})',
  noProcessOutput: 'No process output was recorded.',
  retry: 'Retry',
  remove: 'Remove',
  unsupportedInput: 'is not a supported CUE, GDI, ISO, or CHD source',
  inputUnreadable: 'cannot be read',
  inputNotFound: 'could not be found',
  moreIssue: ', plus {count} more issue.',
  moreIssues: ', plus {count} more issues.',
  sourceInspected: '{count} source set inspected.',
  sourcesInspected: '{count} source sets inspected.',
  destinationConflict:
    'Another queued job already uses this destination. Choose a different name or folder.',
  addedToQueue: '{name} was added to the queue.',
  nativeFilesOnly: 'Native file selection is available in the desktop app.',
  nativeDestinationOnly: 'Native destination selection is available in the desktop app.',
  addFolderDialog: 'Add a folder to Hunk',
  addImagesDialog: 'Add disc images to Hunk',
  opticalImages: 'Optical images',
  destinationDialog: 'Choose a destination folder',
} as const;

export type TranslationKey = keyof typeof en;

const de: Record<TranslationKey, string> = {
  appTitle: 'Hunk — Werkbank für optische Medien',
  primaryNavigation: 'Hauptnavigation',
  workspaceNavigation: 'Arbeitsbereich',
  workbench: 'Werkbank',
  history: 'Verlauf',
  localOnly: 'Nur lokal',
  language: 'Sprache',
  english: 'English',
  german: 'Deutsch',
  localOpticalTools: 'Lokale Werkzeuge für optische Medien',
  ready: 'Bereit',
  needsAttention: 'Prüfung nötig',
  sourcesSafety:
    'Quelldateien bleiben unverändert. Hunk lädt keine Medien hoch und überschreibt keine Ausgaben.',
  images: 'Images',
  folder: 'Ordner',
  sourceDetails: 'Quelldetails',
  closeDetails: 'Quelldetails schließen',
  dismissMessage: 'Meldung schließen',
  activeJob: 'Aktiver Auftrag',
  closeQuestion: 'Verarbeitung stoppen und Hunk schließen?',
  closeExplanation:
    'Der aktive chdman-Prozess wird abgebrochen. Quelldateien und vorhandene Ausgaben bleiben unverändert.',
  keepWorking: 'Weiterarbeiten',
  cancelAndClose: 'Auftrag abbrechen und schließen',
  importedSets: 'Importierte Sets',
  sources: 'Quellen',
  removeSource: '{name} entfernen',
  bringInMedia: 'Optische Medien hinzufügen',
  dropDiscSet: 'Disc-Set hier ablegen.',
  importExplanation:
    'CUE, GDI, ISO und CHD bleiben auf diesem Computer. Referenzierte Tracks werden automatisch gruppiert.',
  inspecting: 'Wird geprüft…',
  chooseImages: 'Images auswählen',
  chooseFolder: 'Ordner auswählen',
  releaseToInspect: 'Zum Prüfen loslassen',
  selectedSource: 'Ausgewählte Quelle',
  track: 'Track',
  tracks: 'Tracks',
  trackDescription: 'Track {number}: {kind}',
  progressComplete: '{value} % abgeschlossen',
  trackLegend: 'Track-Legende',
  data: 'Daten',
  audio: 'Audio',
  subchannel: 'Subchannel',
  unknown: 'Unbekannt',
  cannotQueue: 'Diese Quelle kann nicht eingereiht werden.',
  media: 'Medium',
  sourceSize: 'Quellgröße',
  dependencies: 'Abhängigkeiten',
  location: 'Speicherort',
  isoQuestion: 'Welche Art Disc enthält dieses ISO?',
  isoExplanation:
    'Die ISO-Größe bestimmt den Medientyp nicht zuverlässig. Bitte ausdrücklich auswählen.',
  chooseAction: 'Aktion auswählen',
  chooseMedia: 'CD oder DVD auswählen',
  chooseMediaToContinue: 'CD oder DVD auswählen, um fortzufahren.',
  destination: 'Ziel',
  chooseDestination: 'Zielordner auswählen',
  neverOverwrite: 'Vorhandene Dateien werden niemals überschrieben.',
  advancedOptions: 'Erweiterte Optionen',
  splitBin: 'Separate BIN-Datei pro Track',
  processors: 'Prozessoren',
  automatic: 'Automatisch',
  hunkSize: 'Hunk-Größe (Bytes)',
  chdmanDefault: 'chdman-Standard',
  readOnlyAction: 'Diese Aktion liest die CHD, ohne sie zu verändern.',
  safeCreateAction:
    'Die Quelle bleibt unverändert. Neue CHDs werden vor der Veröffentlichung geprüft.',
  createCd: 'CD-Image erstellen',
  createDvd: 'DVD-Image erstellen',
  extractCd: 'CD-Image extrahieren',
  extractDvd: 'DVD-Image extrahieren',
  verify: 'Integrität prüfen',
  info: 'Informationen lesen',
  missingReference: 'Eine referenzierte Track-Datei fehlt.',
  duplicateReference: 'Eine Track-Datei wird mehrfach referenziert.',
  escapingReference: 'Eine Track-Referenz zeigt aus dem Ordner der Beschreibungsdatei heraus.',
  unreadableReference: 'Eine referenzierte Track-Datei kann nicht gelesen werden.',
  unreadablePrimary: 'Die Quelldatei kann nicht gelesen werden.',
  malformedDescriptor: 'Die Track-Beschreibungsdatei ist fehlerhaft.',
  duplicateTrack: 'Eine Track-Nummer wird mehrfach verwendet.',
  trackCountMismatch: 'Die angegebene und die gefundene Track-Anzahl stimmen nicht überein.',
  line: 'Zeile {number}.',
  serialProcessing: 'Serielle Verarbeitung',
  queue: 'Warteschlange',
  resume: 'Fortsetzen',
  pause: 'Pausieren',
  noQueuedJobs: 'Keine Aufträge eingereiht',
  queueEmptyExplanation: 'Aktion für eine Quelle auswählen. Aufträge laufen sicher nacheinander.',
  queuePaused: 'Die Warteschlange ist pausiert. Der aktive Auftrag darf beendet werden.',
  retryJob: 'Auftrag erneut versuchen',
  cancelJob: 'Auftrag abbrechen',
  removeJob: 'Auftrag entfernen',
  queued: 'Eingereiht',
  preflight: 'Vorprüfung',
  running: 'Wird ausgeführt',
  verifying: 'Wird geprüft',
  completed: 'Abgeschlossen',
  failed: 'Fehlgeschlagen',
  cancelled: 'Abgebrochen',
  interrupted: 'Unterbrochen',
  blocked: 'Blockiert',
  waitingInQueue: 'Wartet in der seriellen Warteschlange',
  interruptedPreviously: 'Beim letzten Beenden von Hunk unterbrochen',
  cancelledBeforeProcessing: 'Vor der Verarbeitung abgebrochen',
  checkingSource: 'Quelle und Ziel werden geprüft',
  fullyVerifying: 'Temporäre CHD wird vollständig geprüft',
  preflightBlocked: 'Die Vorprüfung hat diesen Auftrag blockiert',
  chdmanFailed: 'chdman konnte den Auftrag nicht abschließen',
  latestJobs: 'Letzte 100 Aufträge',
  record: 'Eintrag',
  records: 'Einträge',
  historyEmpty: 'Abgeschlossene Aufträge erscheinen hier.',
  historyExplanation:
    'Jeder Eintrag enthält Pfade, Größenersparnis, Zeitangaben, Status und ein begrenztes Prozessprotokoll.',
  openWorkbench: 'Werkbank öffnen',
  input: 'Eingabe',
  output: 'Ausgabe',
  noOutput: 'Keine Ausgabe',
  size: 'Größe',
  started: 'Gestartet',
  chdInformation: 'CHD-Informationen',
  version: 'Version',
  logical: 'Logisch',
  compressed: 'Komprimiert',
  codecs: 'Codecs',
  none: 'Keine',
  processLog: 'Prozessprotokoll ({count})',
  noProcessOutput: 'Keine Prozessausgabe aufgezeichnet.',
  retry: 'Erneut versuchen',
  remove: 'Entfernen',
  unsupportedInput: 'ist keine unterstützte CUE-, GDI-, ISO- oder CHD-Quelle',
  inputUnreadable: 'kann nicht gelesen werden',
  inputNotFound: 'wurde nicht gefunden',
  moreIssue: ', außerdem besteht {count} weiteres Problem.',
  moreIssues: ', außerdem bestehen {count} weitere Probleme.',
  sourceInspected: '{count} Quell-Set geprüft.',
  sourcesInspected: '{count} Quell-Sets geprüft.',
  destinationConflict:
    'Ein anderer eingereihter Auftrag verwendet dieses Ziel bereits. Bitte einen anderen Namen oder Ordner wählen.',
  addedToQueue: '{name} wurde in die Warteschlange eingereiht.',
  nativeFilesOnly: 'Die native Dateiauswahl ist in der Desktop-App verfügbar.',
  nativeDestinationOnly: 'Die native Zielauswahl ist in der Desktop-App verfügbar.',
  addFolderDialog: 'Ordner zu Hunk hinzufügen',
  addImagesDialog: 'Disc-Images zu Hunk hinzufügen',
  opticalImages: 'Optische Images',
  destinationDialog: 'Zielordner auswählen',
};

export const dictionaries: Record<Locale, Record<TranslationKey, string>> = { en, de };

export function detectLocale(language = globalThis.navigator?.language): Locale {
  return language?.toLowerCase().startsWith('de') ? 'de' : 'en';
}

export function t(locale: Locale, key: TranslationKey, parameters: Parameters = {}): string {
  return Object.entries(parameters).reduce(
    (message, [name, value]) => message.replaceAll(`{${name}}`, String(value)),
    dictionaries[locale][key],
  );
}

export function operationLabel(locale: Locale, operation: Operation): string {
  return t(locale, operation);
}

export function problemMessage(locale: Locale, problem: ValidationProblem): string {
  const reference = problem.reference ? ` ${problem.reference}` : '';
  const location = problem.line ? ` ${t(locale, 'line', { number: problem.line })}` : '';
  return `${t(locale, problem.kind)}${reference}${location}`;
}

export function mediaLabel(locale: Locale, kind: MediaKind): string {
  return kind === 'unknownOptical' ? t(locale, 'chooseMedia') : kind.toUpperCase();
}

export function trackLabel(locale: Locale, kind: TrackKind): string {
  return t(locale, kind);
}

export function jobStateLabel(locale: Locale, state: JobState): string {
  return t(locale, state);
}

const jobMessageKeys: Partial<Record<string, TranslationKey>> = {
  'Waiting in the serial queue': 'waitingInQueue',
  'Interrupted when Hunk last stopped': 'interruptedPreviously',
  'Cancelled before processing': 'cancelledBeforeProcessing',
  'Checking source and destination': 'checkingSource',
  'Fully verifying the temporary CHD': 'fullyVerifying',
  Cancelled: 'cancelled',
  Completed: 'completed',
  'Preflight blocked this job': 'preflightBlocked',
  'chdman could not complete the job': 'chdmanFailed',
};

export function localizeJobMessage(locale: Locale, message: string): string {
  const key = jobMessageKeys[message];
  return key ? t(locale, key) : message;
}

export function formatBytes(locale: Locale, bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  const maximumFractionDigits = value >= 100 || unit === 0 ? 0 : value >= 10 ? 1 : 2;
  return `${new Intl.NumberFormat(locale, { maximumFractionDigits }).format(value)} ${units[unit]}`;
}

export function formatNumber(locale: Locale, value: number): string {
  return new Intl.NumberFormat(locale).format(value);
}

export function formatDate(locale: Locale, timestamp: number | null): string {
  if (timestamp === null) return '—';
  return new Intl.DateTimeFormat(locale, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(timestamp));
}
