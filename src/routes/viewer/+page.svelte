<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import type { ImageEntry } from '$lib/types';

  // ---- state ----
  let allEntries = $state<ImageEntry[]>([]);
  let localClasses = $state<string[]>([]);
  let splits = $state<string[]>([]);

  let filterClass = $state('');
  let filterSplit = $state('');

  let loading = $state(false);
  let error = $state('');

  let currentIndex = $state(0);
  let canvas = $state<HTMLCanvasElement>(null!);

  // ---- derived ----
  const filtered = $derived(
    allEntries.filter((e) => {
      if (filterClass && e.class_name !== filterClass) return false;
      if (filterSplit && e.split !== filterSplit) return false;
      return true;
    })
  );

  const current = $derived(filtered[currentIndex] ?? null);

  // ---- lifecycle ----
  onMount(async () => {
    await refreshIndex();
  });

  // Re-draw whenever current entry changes
  $effect(() => {
    if (current) {
      drawImage(current);
    }
  });

  // ---- helpers ----
  async function refreshIndex() {
    loading = true;
    error = '';
    try {
      allEntries = await invoke<ImageEntry[]>('index_dataset', { datasetDir: null });
      localClasses = [...new Set(allEntries.map((e) => e.class_name))].sort();
      splits = [...new Set(allEntries.map((e) => e.split))].sort();
      currentIndex = 0;
    } catch (e: any) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function prev() {
    if (currentIndex > 0) currentIndex--;
  }

  function next() {
    if (currentIndex < filtered.length - 1) currentIndex++;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowRight' || e.key === 'd') next();
    if (e.key === 'ArrowLeft' || e.key === 'a') prev();
  }

  // Colour palette for up to 20 classes
  const COLOURS = [
    '#48bb78', '#63b3ed', '#f6e05e', '#fc8181', '#b794f4',
    '#f6ad55', '#76e4f7', '#fbb6ce', '#9ae6b4', '#feb2b2',
    '#90cdf4', '#faf089', '#e9d8fd', '#bee3f8', '#fed7d7',
    '#c6f6d5', '#feebc8', '#b2f5ea', '#e9d8fd', '#fed7e2',
  ];
  const colourCache = new Map<string, string>();
  function colourFor(className: string): string {
    if (!colourCache.has(className)) {
      colourCache.set(className, COLOURS[colourCache.size % COLOURS.length]);
    }
    return colourCache.get(className)!;
  }

  async function drawImage(entry: ImageEntry) {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Use Tauri's convertFileSrc for cross-platform local asset URL conversion
    const safeUrl = convertFileSrc(entry.path);

    const img = new Image();
    img.onload = () => {
      canvas.width = img.width;
      canvas.height = img.height;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(img, 0, 0);

      // Draw bounding boxes
      for (const box of entry.boxes) {
        const colour = colourFor(String(box.class_idx));
        const x1 = (box.cx - box.w / 2) * img.width;
        const y1 = (box.cy - box.h / 2) * img.height;
        const bw = box.w * img.width;
        const bh = box.h * img.height;

        ctx.strokeStyle = colour;
        ctx.lineWidth = 2;
        ctx.strokeRect(x1, y1, bw, bh);

        // Label background + text
        const label = `class ${box.class_idx}`;
        ctx.font = '13px Inter, sans-serif';
        const tw = ctx.measureText(label).width;
        ctx.fillStyle = colour;
        ctx.fillRect(x1, y1 - 18, tw + 8, 18);
        ctx.fillStyle = '#000';
        ctx.fillText(label, x1 + 4, y1 - 4);
      }
    };
    img.onerror = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#2d3748';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#a0aec0';
      ctx.font = '14px sans-serif';
      ctx.fillText('Image not available', 20, 40);
    };
    img.src = safeUrl;
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="page">
  <h1>Dataset Viewer</h1>
  <p class="subtitle">
    Browse locally downloaded images. Press <kbd>A</kbd> / <kbd>D</kbd> or
    use the arrows to navigate.
  </p>

  {#if error}
    <div class="alert error">{error}</div>
  {/if}

  <div class="toolbar">
    <label class="filter-label">
      Class
      <select bind:value={filterClass} class="select" onchange={() => (currentIndex = 0)}>
        <option value="">All</option>
        {#each localClasses as cls}
          <option value={cls}>{cls}</option>
        {/each}
      </select>
    </label>

    <label class="filter-label">
      Split
      <select bind:value={filterSplit} class="select" onchange={() => (currentIndex = 0)}>
        <option value="">All</option>
        {#each splits as s}
          <option value={s}>{s}</option>
        {/each}
      </select>
    </label>

    <button class="btn secondary" onclick={refreshIndex} disabled={loading}>
      {loading ? 'Scanning…' : '🔄 Refresh'}
    </button>

    {#if filtered.length > 0}
      <span class="count">{currentIndex + 1} / {filtered.length}</span>
    {/if}
  </div>

  {#if loading}
    <p class="muted">Scanning dataset directory…</p>
  {:else if filtered.length === 0}
    <div class="empty">
      <span class="empty-icon">🗂️</span>
      <p>No images found. Download some first via the <a href="/download">Download</a> page.</p>
    </div>
  {:else}
    <div class="viewer">
      <div class="canvas-wrap">
        <canvas bind:this={canvas}></canvas>
      </div>

      <div class="info-panel">
        {#if current}
          <div class="info-field"><span class="info-key">ID</span><code>{current.image_id}</code></div>
          <div class="info-field"><span class="info-key">Class</span>{current.class_name}</div>
          <div class="info-field"><span class="info-key">Split</span>{current.split}</div>
          <div class="info-field"><span class="info-key">Boxes</span>{current.boxes.length}</div>
          {#if current.label_path}
            <div class="info-field">
              <span class="info-key">Label</span>
              <code class="path">{current.label_path}</code>
            </div>
          {:else}
            <div class="info-field"><span class="info-key">Label</span><span class="muted">none</span></div>
          {/if}
        {/if}

        <div class="nav-btns">
          <button class="nav-btn" onclick={prev} disabled={currentIndex === 0}>◀ Prev</button>
          <button class="nav-btn" onclick={next} disabled={currentIndex >= filtered.length - 1}>Next ▶</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .page { max-width: 1100px; }
  h1 { font-size: 1.7rem; font-weight: 800; color: #90cdf4; margin-bottom: 6px; }
  .subtitle { color: #718096; font-size: 0.88rem; margin-bottom: 20px; }
  kbd {
    font-size: 0.78rem;
    background: #2d3748;
    border: 1px solid #4a5568;
    border-radius: 4px;
    padding: 1px 5px;
    color: #e2e8f0;
  }

  .alert {
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 0.85rem;
    margin-bottom: 16px;
  }
  .error { background: #2d1b1b; color: #fc8181; border: 1px solid #742a2a; }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 16px;
  }

  .filter-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.82rem;
    color: #a0aec0;
  }

  .select {
    background: #0f1117;
    border: 1px solid #2d3748;
    border-radius: 6px;
    color: #e2e8f0;
    padding: 5px 8px;
    font-size: 0.82rem;
  }

  .count { margin-left: auto; color: #4a5568; font-size: 0.8rem; }

  .muted { color: #4a5568; font-size: 0.85rem; }

  .empty { text-align: center; padding: 60px 20px; color: #4a5568; }
  .empty-icon { font-size: 3rem; display: block; margin-bottom: 12px; }
  .empty a { color: #63b3ed; }

  .viewer {
    display: grid;
    grid-template-columns: 1fr 260px;
    gap: 20px;
    align-items: start;
  }

  .canvas-wrap {
    background: #0a0e14;
    border: 1px solid #2d3748;
    border-radius: 10px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 300px;
  }

  canvas {
    max-width: 100%;
    max-height: 600px;
    display: block;
  }

  .info-panel {
    background: #161b27;
    border: 1px solid #2d3748;
    border-radius: 10px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .info-field {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .info-key {
    font-size: 0.68rem;
    font-weight: 700;
    text-transform: uppercase;
    color: #4a5568;
    letter-spacing: 0.05em;
  }

  code {
    font-size: 0.78rem;
    color: #68d391;
    word-break: break-all;
  }

  .path { color: #a0aec0; font-size: 0.7rem; }

  .nav-btns { display: flex; gap: 8px; margin-top: 8px; }

  .nav-btn {
    flex: 1;
    padding: 7px;
    background: #2d3748;
    border: none;
    border-radius: 6px;
    color: #e2e8f0;
    font-size: 0.82rem;
    cursor: pointer;
  }
  .nav-btn:hover:not(:disabled) { background: #4a5568; }
  .nav-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  .btn {
    padding: 7px 14px;
    border-radius: 8px;
    border: none;
    font-size: 0.83rem;
    font-weight: 600;
    cursor: pointer;
  }
  .btn.secondary { background: #2d3748; color: #e2e8f0; }
  .btn.secondary:hover:not(:disabled) { background: #4a5568; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
