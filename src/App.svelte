<script lang="ts">
  import { onMount } from 'svelte';
  import hunkMark from '../assets/hunk-mark.svg';
  import HistoryView from './lib/components/HistoryView.svelte';
  import ImportSurface from './lib/components/ImportSurface.svelte';
  import Inspector from './lib/components/Inspector.svelte';
  import QueuePanel from './lib/components/QueuePanel.svelte';
  import SourceList from './lib/components/SourceList.svelte';
  import { chooseDestination, chooseSources, discover, listenForDroppedPaths } from './lib/backend';
  import { basename, defaultDestination, dirname, operationsFor } from './lib/presentation';
  import type {
    AdvancedOptions,
    DiscoveryIssue,
    HistoryItem,
    MediaKind,
    Operation,
    QueueItem,
    SourceSet,
  } from './lib/types';

  let view: 'workbench' | 'history' = 'workbench';
  let sources: SourceSet[] = [];
  let selectedIndex = 0;
  let isoChoices: Record<string, MediaKind> = {};
  let operation: Operation | undefined;
  let destination: string | null = null;
  let importing = false;
  let hovering = false;
  let inspectorOpen = false;
  let notice: { tone: 'error' | 'info'; text: string } | null = null;
  let queue: QueueItem[] = [];
  let history: HistoryItem[] = [];
  let advanced: AdvancedOptions = { splitBin: false, processors: null, hunkSize: null };

  $: selected = sources[selectedIndex];
  $: selectedMedia = selected ? isoChoices[selected.primaryFile] : undefined;
  $: operations = selected ? operationsFor(selected, selectedMedia) : [];

  onMount(() => {
    let dispose: () => void = () => {};
    listenForDroppedPaths(
      (active) => (hovering = active),
      (paths) => void importPaths(paths),
    )
      .then((unlisten) => (dispose = unlisten))
      .catch((error: unknown) => showError(error));
    return () => dispose();
  });

  function showError(error: unknown) {
    notice = {
      tone: 'error',
      text: error instanceof Error ? error.message : String(error),
    };
  }

  function reportIssues(issues: DiscoveryIssue[]) {
    if (!issues.length) return;
    const first = issues[0];
    const reason =
      first.kind === 'unsupportedInput'
        ? 'is not a supported CUE, GDI, ISO, or CHD source'
        : first.kind === 'inputUnreadable'
          ? 'cannot be read'
          : 'could not be found';
    notice = {
      tone: 'error',
      text: `${basename(first.path)} ${reason}${issues.length > 1 ? `, plus ${issues.length - 1} more issue${issues.length === 2 ? '' : 's'}.` : '.'}`,
    };
  }

  async function importPaths(paths: string[]) {
    if (!paths.length || importing) return;
    importing = true;
    notice = null;
    try {
      const report = await discover(paths);
      const merged = new Map(sources.map((source) => [source.primaryFile, source]));
      report.sourceSets.forEach((source) => merged.set(source.primaryFile, source));
      sources = [...merged.values()];
      if (report.sourceSets.length) {
        selectedIndex = sources.findIndex(
          (source) => source.primaryFile === report.sourceSets[0].primaryFile,
        );
        resetWorkflow();
        inspectorOpen = true;
        notice = {
          tone: 'info',
          text: `${report.sourceSets.length} source set${report.sourceSets.length === 1 ? '' : 's'} inspected.`,
        };
      }
      reportIssues(report.issues);
    } catch (error) {
      showError(error);
    } finally {
      importing = false;
    }
  }

  async function openPicker(directory: boolean) {
    try {
      await importPaths(await chooseSources(directory));
    } catch (error) {
      showError(error);
    }
  }

  function resetWorkflow() {
    operation = undefined;
    destination = null;
    advanced = { splitBin: false, processors: null, hunkSize: null };
  }

  function selectSource(index: number) {
    selectedIndex = index;
    resetWorkflow();
    inspectorOpen = true;
  }

  function removeSource(index: number) {
    sources = sources.filter((_, itemIndex) => itemIndex !== index);
    selectedIndex = Math.min(selectedIndex, Math.max(0, sources.length - 1));
    resetWorkflow();
  }

  function selectMedia(kind: MediaKind) {
    if (!selected) return;
    isoChoices = { ...isoChoices, [selected.primaryFile]: kind };
    const nextOperations = operationsFor(selected, kind);
    operation = nextOperations[0];
    destination = operation ? defaultDestination(selected, operation, advanced.splitBin) : null;
  }

  function selectOperation(next: Operation) {
    operation = next;
    destination = selected ? defaultDestination(selected, next, advanced.splitBin) : null;
  }

  async function selectDestination() {
    if (!selected || !operation) return;
    try {
      const folder = await chooseDestination(
        destination ? dirname(destination) : dirname(selected.primaryFile),
      );
      if (!folder) return;
      const fileName = basename(defaultDestination(selected, operation, advanced.splitBin) ?? '');
      const separator = folder.includes('\\') && !folder.includes('/') ? '\\' : '/';
      destination = `${folder.replace(/[\\/]$/, '')}${separator}${fileName}`;
    } catch (error) {
      showError(error);
    }
  }

  function queueOperation() {
    if (!selected || !operation) return;
    const collision =
      destination &&
      queue.some(
        (item) => item.destination?.toLocaleLowerCase() === destination?.toLocaleLowerCase(),
      );
    if (collision) {
      notice = {
        tone: 'error',
        text: 'Another queued job already uses this destination. Choose a different name or folder.',
      };
      return;
    }
    queue = [
      ...queue,
      {
        id: crypto.randomUUID(),
        source: selected,
        operation,
        destination,
        status: 'queued',
        message: 'Ready for serial processing',
        createdAt: new Date(),
      },
    ];
    notice = { tone: 'info', text: `${basename(selected.primaryFile)} was added to the queue.` };
  }

  function removeHistory(id: string) {
    history = history.filter((item) => item.id !== id);
  }
</script>

<svelte:head><title>Hunk — Optical workbench</title></svelte:head>

<div class="app-shell">
  <aside class="rail" aria-label="Primary navigation">
    <div class="brand" aria-label="Hunk">
      <img src={hunkMark} alt="" /><span>Hunk</span>
    </div>
    <nav aria-label="Workspace">
      <button
        type="button"
        class:active={view === 'workbench'}
        aria-current={view === 'workbench' ? 'page' : undefined}
        onclick={() => (view = 'workbench')}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M4 5h16v14H4zM8 9h8M8 13h5" /></svg>
        <span>Workbench</span>
      </button>
      <button
        type="button"
        class:active={view === 'history'}
        aria-current={view === 'history' ? 'page' : undefined}
        onclick={() => (view = 'history')}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24"
          ><path d="M5 4v16h14V4M8 8h8M8 12h8M8 16h5" /></svg
        >
        <span>History</span>
      </button>
    </nav>
    <div class="rail-status"><span></span><small>Local only</small></div>
    <span class="version">Workbench · 0.0.0</span>
  </aside>

  <main>
    {#if view === 'history'}
      <HistoryView
        items={history}
        onWorkbench={() => (view = 'workbench')}
        onRemove={removeHistory}
      />
    {:else if sources.length === 0}
      <div class="empty-workbench">
        <header class="topbar">
          <div>
            <p class="eyebrow">Local optical media tools</p>
            <h1>Workbench</h1>
          </div>
          <span class="app-status"><span></span> Ready</span>
        </header>
        <ImportSurface
          {importing}
          {hovering}
          onFiles={() => void openPicker(false)}
          onFolder={() => void openPicker(true)}
        />
        <footer>
          <span></span>
          <p>Sources stay untouched. Hunk never uploads media or overwrites output.</p>
        </footer>
      </div>
    {:else}
      <div class="workbench-view">
        <header class="workbench-bar">
          <div>
            <p class="eyebrow">Local optical media tools</p>
            <h1>Workbench</h1>
          </div>
          <div class="toolbar">
            <button
              class="secondary"
              type="button"
              onclick={() => void openPicker(false)}
              disabled={importing}>＋ Images</button
            >
            <button
              class="secondary"
              type="button"
              onclick={() => void openPicker(true)}
              disabled={importing}>＋ Folder</button
            >
            <button class="details-toggle" type="button" onclick={() => (inspectorOpen = true)}
              >Source details</button
            >
          </div>
        </header>

        {#if notice}
          <div
            class:error={notice.tone === 'error'}
            class="notice"
            role={notice.tone === 'error' ? 'alert' : 'status'}
          >
            <span>{notice.tone === 'error' ? '!' : '✓'}</span>
            <p>{notice.text}</p>
            <button type="button" aria-label="Dismiss message" onclick={() => (notice = null)}
              >×</button
            >
          </div>
        {/if}

        <div class="workbench-grid">
          <SourceList
            {sources}
            selected={selectedIndex}
            onSelect={selectSource}
            onRemove={removeSource}
          />
          {#if selected}
            <div class:open={inspectorOpen} class="inspector-drawer">
              <button class="drawer-close" type="button" onclick={() => (inspectorOpen = false)}
                >Close details</button
              >
              <Inspector
                source={selected}
                mediaChoice={selectedMedia}
                {operations}
                {operation}
                {destination}
                {advanced}
                onMediaChoice={selectMedia}
                onOperation={selectOperation}
                onDestination={(value) => (destination = value)}
                onChooseDestination={() => void selectDestination()}
                onQueue={queueOperation}
              />
            </div>
          {/if}
          <QueuePanel
            items={queue}
            onRemove={(id) => (queue = queue.filter((item) => item.id !== id))}
          />
        </div>
        {#if inspectorOpen}<button
            class="drawer-backdrop"
            aria-label="Close source details"
            onclick={() => (inspectorOpen = false)}
          ></button>{/if}
      </div>
    {/if}
  </main>
</div>
