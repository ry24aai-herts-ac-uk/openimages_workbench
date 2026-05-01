<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let dataDir = $state('');

  onMount(async () => {
    try {
      dataDir = await invoke<string>('get_data_dir');
    } catch (_) {}
  });
</script>

<div class="home">
  <h1>Open Images Workbench</h1>
  <p class="subtitle">
    A desktop workbench for downloading, exploring, and exporting
    <strong>Open Images</strong> datasets — powered by Rust + Tauri + SvelteKit.
  </p>

  <div class="cards">
    <a href="/download" class="card">
      <span class="card-icon">⬇️</span>
      <h2>Download Dataset</h2>
      <p>
        Select classes, choose a dataset split and annotation filters, then
        fetch images and YOLO-format labels from the Open Images collection.
      </p>
    </a>

    <a href="/jobs" class="card">
      <span class="card-icon">📋</span>
      <h2>Download Jobs</h2>
      <p>
        Monitor active and completed download jobs with live progress bars
        and per-image status updates.
      </p>
    </a>

    <a href="/viewer" class="card">
      <span class="card-icon">🖼️</span>
      <h2>Dataset Viewer</h2>
      <p>
        Browse locally downloaded images with bounding-box overlays rendered
        from the YOLO label files.
      </p>
    </a>
  </div>

  {#if dataDir}
    <p class="data-dir">📁 Data directory: <code>{dataDir}</code></p>
  {/if}

  <div class="phase-badge">Phase 1 — Foundation</div>
</div>

<style>
  .home {
    max-width: 900px;
  }

  h1 {
    font-size: 2rem;
    font-weight: 800;
    color: #90cdf4;
    margin-bottom: 10px;
  }

  .subtitle {
    color: #a0aec0;
    font-size: 1rem;
    margin-bottom: 36px;
    line-height: 1.6;
  }

  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 20px;
    margin-bottom: 32px;
  }

  .card {
    background: #161b27;
    border: 1px solid #2d3748;
    border-radius: 12px;
    padding: 24px;
    text-decoration: none;
    color: inherit;
    transition: border-color 0.2s, transform 0.15s;
    display: block;
  }

  .card:hover {
    border-color: #4a90d9;
    transform: translateY(-2px);
  }

  .card-icon {
    font-size: 2rem;
    display: block;
    margin-bottom: 12px;
  }

  .card h2 {
    font-size: 1.1rem;
    font-weight: 700;
    color: #e2e8f0;
    margin-bottom: 8px;
  }

  .card p {
    font-size: 0.85rem;
    color: #718096;
    line-height: 1.6;
  }

  .data-dir {
    font-size: 0.8rem;
    color: #4a5568;
    margin-top: 12px;
  }

  .data-dir code {
    color: #68d391;
    font-size: 0.78rem;
  }

  .phase-badge {
    display: inline-block;
    margin-top: 24px;
    background: #2b4b7e;
    color: #90cdf4;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 99px;
  }
</style>
