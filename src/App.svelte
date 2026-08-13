<script lang="ts">
  import { onMount, tick } from 'svelte';
  import hunkMark from '../assets/hunk-mark.svg';
  import HistoryView from './lib/components/HistoryView.svelte';
  import ImportSurface from './lib/components/ImportSurface.svelte';
  import Inspector from './lib/components/Inspector.svelte';
  import QueuePanel from './lib/components/QueuePanel.svelte';
  import SourceList from './lib/components/SourceList.svelte';
  import {
    cancelJob,
    chooseDestination,
    chooseSources,
    confirmClose,
    discover,
    enqueueJob,
    getHistory,
    getQueue,
    getSettings,
    isDesktop,
    listenForDroppedPaths,
    listenForEngineEvents,
    removeJob,
    retryJob,
    setQueuePaused,
    updateSettings,
  } from './lib/backend';
  import { detectLocale, t, type Locale } from './lib/i18n';
  import { basename, defaultDestination, dirname, operationsFor } from './lib/presentation';
  import type {
    AdvancedOptions,
    DiscoveryIssue,
    JobRecord,
    MediaKind,
    Operation,
    QueueSnapshot,
    Settings,
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
  let closeConfirmation = false;
  let narrowInspector = false;
  let detailsButton: HTMLButtonElement;
  let drawerClose: HTMLButtonElement;
  let notice: { tone: 'error' | 'info'; text: string } | null = null;
  let queue: QueueSnapshot = { paused: false, activeJobId: null, jobs: [] };
  let history: JobRecord[] = [];
  let locale: Locale = detectLocale();
  let settings: Settings = { destinationDirectory: null, locale: null };
  let advanced: AdvancedOptions = { splitBin: false, processors: null, hunkSize: null };

  $: selected = sources[selectedIndex];
  $: selectedMedia = selected ? isoChoices[selected.primaryFile] : undefined;
  $: operations = selected ? operationsFor(selected, selectedMedia) : [];
  $: if (typeof document !== 'undefined') {
    document.documentElement.lang = locale;
    document.title = t(locale, 'appTitle');
  }

  onMount(() => {
    const disposers: Array<() => void> = [];
    const narrowQuery = window.matchMedia('(max-width: 720px)');
    const updateNarrowInspector = () => (narrowInspector = narrowQuery.matches);
    updateNarrowInspector();
    narrowQuery.addEventListener('change', updateNarrowInspector);
    Promise.all([
      listenForDroppedPaths(
        (active) => (hovering = active),
        (paths) => void importPaths(paths),
      ),
      listenForEngineEvents({
        jobChanged: handleJobChanged,
        progressChanged: (id, progress) => {
          queue = {
            ...queue,
            jobs: queue.jobs.map((item) => (item.id === id ? { ...item, progress } : item)),
          };
        },
        queueChanged: (snapshot) => (queue = snapshot),
        closeRequested: () => (closeConfirmation = true),
      }),
    ])
      .then((unlisten) => disposers.push(...unlisten))
      .catch((error: unknown) => showError(error));
    if (isDesktop()) {
      Promise.all([getQueue(), getHistory(), getSettings()])
        .then(([queueState, records, preferences]) => {
          queue = queueState;
          history = records;
          settings = preferences;
          locale = preferences.locale ?? locale;
          records.forEach(applyChdInfo);
        })
        .catch((error: unknown) => showError(error));
    }
    return () => {
      narrowQuery.removeEventListener('change', updateNarrowInspector);
      disposers.forEach((dispose) => dispose());
    };
  });

  function handleJobChanged(record: JobRecord) {
    if (record.chdInfo) applyChdInfo(record);
    if (['completed', 'failed', 'cancelled', 'interrupted'].includes(record.state)) {
      void getHistory()
        .then((records) => (history = records))
        .catch(showError);
    }
  }

  function applyChdInfo(record: JobRecord) {
    const info = record.chdInfo;
    if (!info) return;
    sources = sources.map((source) =>
      source.primaryFile === record.spec.source.primaryFile
        ? {
            ...source,
            mediaKind: info.mediaKind,
            tracks: info.tracks.map((track) => ({
              number: track.number,
              kind: track.kind,
              sourceFile: source.primaryFile,
              startLba: null,
              sectorSize: null,
            })),
          }
        : source,
    );
  }

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
        ? t(locale, 'unsupportedInput')
        : first.kind === 'inputUnreadable'
          ? t(locale, 'inputUnreadable')
          : t(locale, 'inputNotFound');
    const additional = issues.length - 1;
    notice = {
      tone: 'error',
      text: `${basename(first.path)} ${reason}${additional > 0 ? t(locale, additional === 1 ? 'moreIssue' : 'moreIssues', { count: additional }) : '.'}`,
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
        await showInspector(narrowInspector);
        notice = {
          tone: 'info',
          text: t(locale, report.sourceSets.length === 1 ? 'sourceInspected' : 'sourcesInspected', {
            count: report.sourceSets.length,
          }),
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
      await importPaths(await chooseSources(directory, locale));
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
    void showInspector(narrowInspector);
  }

  function removeSource(index: number) {
    sources = sources.filter((_, itemIndex) => itemIndex !== index);
    selectedIndex = Math.min(selectedIndex, Math.max(0, sources.length - 1));
    resetWorkflow();
  }

  async function showInspector(moveFocus = true) {
    inspectorOpen = true;
    if (moveFocus && narrowInspector) {
      await tick();
      drawerClose?.focus();
    }
  }

  async function hideInspector(restoreFocus = true) {
    inspectorOpen = false;
    if (restoreFocus && narrowInspector) {
      await tick();
      detailsButton?.focus();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && narrowInspector && inspectorOpen) {
      event.preventDefault();
      void hideInspector();
    }
  }

  function openModal(node: HTMLDialogElement) {
    node.showModal();
    requestAnimationFrame(() => node.querySelector<HTMLElement>('[data-autofocus]')?.focus());
    return { destroy: () => node.close() };
  }

  async function changeLocale(next: Locale) {
    locale = next;
    notice = null;
    settings = { ...settings, locale: next };
    if (!isDesktop()) return;
    try {
      settings = await updateSettings(settings);
    } catch (error) {
      showError(error);
    }
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
        locale,
        destination
          ? dirname(destination)
          : (settings.destinationDirectory ?? dirname(selected.primaryFile)),
      );
      if (!folder) return;
      const fileName = basename(defaultDestination(selected, operation, advanced.splitBin) ?? '');
      const separator = folder.includes('\\') && !folder.includes('/') ? '\\' : '/';
      destination = `${folder.replace(/[\\/]$/, '')}${separator}${fileName}`;
      settings = await updateSettings({ ...settings, destinationDirectory: folder });
    } catch (error) {
      showError(error);
    }
  }

  async function queueOperation() {
    if (!selected || !operation) return;
    const collision =
      destination &&
      queue.jobs.some(
        (item) => item.spec.destination?.toLocaleLowerCase() === destination?.toLocaleLowerCase(),
      );
    if (collision) {
      notice = {
        tone: 'error',
        text: t(locale, 'destinationConflict'),
      };
      return;
    }
    try {
      await enqueueJob({
        source: selected,
        operation,
        destination,
        options: advanced,
      });
      queue = await getQueue();
      notice = {
        tone: 'info',
        text: t(locale, 'addedToQueue', { name: basename(selected.primaryFile) }),
      };
    } catch (error) {
      showError(error);
    }
  }

  async function pauseQueue(paused: boolean) {
    try {
      queue = await setQueuePaused(paused);
    } catch (error) {
      showError(error);
    }
  }

  async function cancelRecord(id: string) {
    try {
      await cancelJob(id);
      queue = await getQueue();
    } catch (error) {
      showError(error);
    }
  }

  async function retryRecord(id: string) {
    try {
      await retryJob(id);
      queue = await getQueue();
      view = 'workbench';
    } catch (error) {
      showError(error);
    }
  }

  async function removeRecord(id: string) {
    try {
      await removeJob(id);
      queue = await getQueue();
      history = await getHistory();
    } catch (error) {
      showError(error);
    }
  }
</script>

<svelte:head><title>{t(locale, 'appTitle')}</title></svelte:head>
<svelte:window onkeydown={handleKeydown} />

<div class="app-shell">
  <aside
    class="rail"
    aria-label={t(locale, 'primaryNavigation')}
    inert={narrowInspector && inspectorOpen}
  >
    <div class="brand" aria-label="Hunk">
      <img src={hunkMark} alt="" /><span>Hunk</span>
    </div>
    <nav aria-label={t(locale, 'workspaceNavigation')}>
      <button
        type="button"
        class:active={view === 'workbench'}
        aria-current={view === 'workbench' ? 'page' : undefined}
        onclick={() => (view = 'workbench')}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M4 5h16v14H4zM8 9h8M8 13h5" /></svg>
        <span>{t(locale, 'workbench')}</span>
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
        <span>{t(locale, 'history')}</span>
      </button>
    </nav>
    <label class="language-control">
      <span>{t(locale, 'language')}</span>
      <select
        value={locale}
        aria-label={t(locale, 'language')}
        onchange={(event) => void changeLocale(event.currentTarget.value as Locale)}
      >
        <option value="en">EN</option>
        <option value="de">DE</option>
      </select>
    </label>
    <div class="rail-status"><span></span><small>{t(locale, 'localOnly')}</small></div>
    <span class="version">{t(locale, 'workbench')} · 0.1.0</span>
  </aside>

  <main>
    {#if view === 'history'}
      <HistoryView
        items={history}
        {locale}
        onWorkbench={() => (view = 'workbench')}
        onRetry={(id) => void retryRecord(id)}
        onRemove={(id) => void removeRecord(id)}
      />
    {:else if sources.length === 0 && queue.jobs.length === 0}
      <div class="empty-workbench">
        <header class="topbar">
          <div>
            <p class="eyebrow">{t(locale, 'localOpticalTools')}</p>
            <h1>{t(locale, 'workbench')}</h1>
          </div>
          <span class="app-status"><span></span> {t(locale, 'ready')}</span>
        </header>
        <ImportSurface
          {importing}
          {hovering}
          {locale}
          onFiles={() => void openPicker(false)}
          onFolder={() => void openPicker(true)}
        />
        <footer>
          <span></span>
          <p>{t(locale, 'sourcesSafety')}</p>
        </footer>
      </div>
    {:else}
      <div class="workbench-view">
        <header class="workbench-bar" inert={narrowInspector && inspectorOpen}>
          <div>
            <p class="eyebrow">{t(locale, 'localOpticalTools')}</p>
            <h1>{t(locale, 'workbench')}</h1>
          </div>
          <div class="toolbar">
            <button
              class="secondary"
              type="button"
              onclick={() => void openPicker(false)}
              disabled={importing}>＋ {t(locale, 'images')}</button
            >
            <button
              class="secondary"
              type="button"
              onclick={() => void openPicker(true)}
              disabled={importing}>＋ {t(locale, 'folder')}</button
            >
            <button
              bind:this={detailsButton}
              class="details-toggle"
              type="button"
              aria-expanded={inspectorOpen}
              aria-controls="source-inspector"
              onclick={() => void showInspector()}>{t(locale, 'sourceDetails')}</button
            >
          </div>
        </header>

        {#if notice}
          <div
            class:error={notice.tone === 'error'}
            class="notice"
            role={notice.tone === 'error' ? 'alert' : 'status'}
            inert={narrowInspector && inspectorOpen}
          >
            <span>{notice.tone === 'error' ? '!' : '✓'}</span>
            <p>{notice.text}</p>
            <button
              type="button"
              aria-label={t(locale, 'dismissMessage')}
              onclick={() => (notice = null)}>×</button
            >
          </div>
        {/if}

        <div class="workbench-grid">
          <SourceList
            {sources}
            selected={selectedIndex}
            {locale}
            inactive={narrowInspector && inspectorOpen}
            onSelect={selectSource}
            onRemove={removeSource}
          />
          {#if selected}
            <div
              id="source-inspector"
              class:open={inspectorOpen}
              class="inspector-drawer"
              inert={narrowInspector && !inspectorOpen}
              aria-hidden={narrowInspector && !inspectorOpen ? 'true' : undefined}
              role={narrowInspector ? 'dialog' : undefined}
              aria-modal={narrowInspector ? 'true' : undefined}
              aria-labelledby={narrowInspector ? 'inspector-title' : undefined}
            >
              <button
                bind:this={drawerClose}
                class="drawer-close"
                type="button"
                onclick={() => void hideInspector()}>{t(locale, 'closeDetails')}</button
              >
              <Inspector
                source={selected}
                {locale}
                mediaChoice={selectedMedia}
                {operations}
                {operation}
                {destination}
                {advanced}
                onMediaChoice={selectMedia}
                onOperation={selectOperation}
                onDestination={(value) => (destination = value)}
                onChooseDestination={() => void selectDestination()}
                onQueue={() => void queueOperation()}
              />
            </div>
          {:else}
            <div class="queue-import">
              <ImportSurface
                {importing}
                {hovering}
                {locale}
                onFiles={() => void openPicker(false)}
                onFolder={() => void openPicker(true)}
              />
            </div>
          {/if}
          <QueuePanel
            items={queue.jobs}
            paused={queue.paused}
            activeJobId={queue.activeJobId}
            {locale}
            inactive={narrowInspector && inspectorOpen}
            onPause={(paused) => void pauseQueue(paused)}
            onCancel={(id) => void cancelRecord(id)}
            onRetry={(id) => void retryRecord(id)}
            onRemove={(id) => void removeRecord(id)}
          />
        </div>
        {#if inspectorOpen}<button
            class="drawer-backdrop"
            tabindex="-1"
            aria-label={t(locale, 'closeDetails')}
            onclick={() => void hideInspector()}
          ></button>{/if}
      </div>
    {/if}
  </main>
</div>

{#if closeConfirmation}
  <dialog
    use:openModal
    class="close-dialog"
    aria-labelledby="close-title"
    oncancel={(event) => {
      event.preventDefault();
      closeConfirmation = false;
    }}
  >
    <p class="section-label">{t(locale, 'activeJob')}</p>
    <h2 id="close-title">{t(locale, 'closeQuestion')}</h2>
    <p>{t(locale, 'closeExplanation')}</p>
    <div>
      <button
        data-autofocus
        class="secondary"
        type="button"
        onclick={() => (closeConfirmation = false)}>{t(locale, 'keepWorking')}</button
      >
      <button class="danger-button" type="button" onclick={() => void confirmClose()}
        >{t(locale, 'cancelAndClose')}</button
      >
    </div>
  </dialog>
{/if}
