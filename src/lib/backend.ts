import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  DiscoveryReport,
  JobProgress,
  JobRecord,
  JobSpec,
  QueueSnapshot,
  Settings,
} from './types';

export function isDesktop(): boolean {
  return '__TAURI_INTERNALS__' in window;
}

export async function discover(paths: string[]): Promise<DiscoveryReport> {
  if (!isDesktop()) {
    return { sourceSets: [], issues: paths.map((path) => ({ kind: 'inputNotFound', path })) };
  }
  return invoke<DiscoveryReport>('discover_sources', { paths });
}

export async function chooseSources(directory: boolean): Promise<string[]> {
  if (!isDesktop()) throw new Error('Native file selection is available in the desktop app.');
  const selection = await open({
    directory,
    multiple: !directory,
    title: directory ? 'Add a folder to Hunk' : 'Add disc images to Hunk',
    filters: directory
      ? undefined
      : [{ name: 'Optical images', extensions: ['cue', 'gdi', 'iso', 'chd'] }],
  });
  if (!selection) return [];
  return Array.isArray(selection) ? selection : [selection];
}

export async function chooseDestination(defaultPath?: string): Promise<string | null> {
  if (!isDesktop())
    throw new Error('Native destination selection is available in the desktop app.');
  const selection = await open({
    directory: true,
    multiple: false,
    defaultPath,
    title: 'Choose a destination folder',
  });
  return typeof selection === 'string' ? selection : null;
}

export async function listenForDroppedPaths(
  onHover: (hovering: boolean) => void,
  onDrop: (paths: string[]) => void,
): Promise<() => void> {
  if (!isDesktop()) return () => undefined;
  return getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === 'over') onHover(true);
    if (event.payload.type === 'leave') onHover(false);
    if (event.payload.type === 'drop') {
      onHover(false);
      onDrop(event.payload.paths);
    }
  });
}

export async function enqueueJob(jobSpec: JobSpec): Promise<JobRecord> {
  return invoke<JobRecord>('enqueue_job', { jobSpec });
}

export async function getQueue(): Promise<QueueSnapshot> {
  return invoke<QueueSnapshot>('get_queue');
}

export async function setQueuePaused(paused: boolean): Promise<QueueSnapshot> {
  return invoke<QueueSnapshot>('set_queue_paused', { paused });
}

export async function cancelJob(id: string): Promise<JobRecord> {
  return invoke<JobRecord>('cancel_job', { id });
}

export async function retryJob(id: string): Promise<JobRecord> {
  return invoke<JobRecord>('retry_job', { id });
}

export async function removeJob(id: string): Promise<void> {
  await invoke('remove_job', { id });
}

export async function getHistory(): Promise<JobRecord[]> {
  return invoke<JobRecord[]>('get_history');
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings');
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  return invoke<Settings>('update_settings', { settings });
}

export async function confirmClose(): Promise<void> {
  await invoke('confirm_close');
}

export interface EngineEventHandlers {
  jobChanged: (record: JobRecord) => void;
  progressChanged: (id: string, progress: JobProgress) => void;
  queueChanged: (snapshot: QueueSnapshot) => void;
  closeRequested: () => void;
}

export async function listenForEngineEvents(handlers: EngineEventHandlers): Promise<() => void> {
  if (!isDesktop()) return () => undefined;
  const unlisten = await Promise.all([
    listen<JobRecord>('job-state', (event) => handlers.jobChanged(event.payload)),
    listen<{ id: string; progress: JobProgress }>('job-progress', (event) =>
      handlers.progressChanged(event.payload.id, event.payload.progress),
    ),
    listen<QueueSnapshot>('queue-state', (event) => handlers.queueChanged(event.payload)),
    listen('close-requested', () => handlers.closeRequested()),
  ]);
  return () => unlisten.forEach((dispose) => dispose());
}
