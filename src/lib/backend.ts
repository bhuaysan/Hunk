import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { open } from '@tauri-apps/plugin-dialog';
import type { DiscoveryReport } from './types';

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
