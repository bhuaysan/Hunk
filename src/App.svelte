<svelte:head><title>Hunk — Optical workbench</title></svelte:head>

<div class="app-shell">
  <aside class="rail" aria-label="Primary navigation">
    <div class="brand" aria-label="Hunk">
      <span class="brand-mark" aria-hidden="true">H</span><span class="brand-name">Hunk</span>
    </div>
    <nav aria-label="Workspace">
      <a class="nav-item active" href="#workbench" aria-current="page">
        <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M4 5h16v14H4zM8 9h8M8 13h5" /></svg>
        <span>Workbench</span>
      </a>
      <span class="nav-item unavailable" aria-disabled="true">
        <svg aria-hidden="true" viewBox="0 0 24 24"
          ><path d="M5 4v16h14V4M8 8h8M8 12h8M8 16h5" /></svg
        >
        <span>History</span>
      </span>
    </nav>
    <span class="version">Foundation · 0.0.0</span>
  </aside>

  <main id="workbench">
    <header class="topbar">
      <div>
        <p class="eyebrow">Local optical media tools</p>
        <h1>Workbench</h1>
      </div>
      <span class="status"><span aria-hidden="true"></span> Ready</span>
    </header>

    <section class="welcome" aria-labelledby="welcome-title">
      <div class="disc" aria-hidden="true"><span></span></div>
      <div class="welcome-copy">
        <p class="section-label">Hunk workspace</p>
        <h2 id="welcome-title">A safe place for every track.</h2>
        <p>The desktop foundation is ready. Source discovery and validation are the next step.</p>
      </div>
      <div class="track-preview" aria-label="Track band preview">
        <div class="track-labels" aria-hidden="true">
          <span>Data</span><span>Audio</span><span>Audio</span>
        </div>
        <div class="track-band" aria-hidden="true">
          <span class="data"></span><span class="audio"></span><span class="audio dim"></span>
        </div>
      </div>
    </section>

    <footer>
      <span aria-hidden="true"></span>
      <p>No files are read, changed, or uploaded from this foundation screen.</p>
    </footer>
  </main>
</div>
