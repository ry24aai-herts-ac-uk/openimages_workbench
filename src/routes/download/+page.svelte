<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import type { ClassEntry, DownloadConfig, Split } from '$lib/types';
  import { jobStore } from '$lib/jobStore';

  // ---- state ----
  let allClasses = $state<ClassEntry[]>([]);
  let classSearch = $state('');
  let selectedClasses = $state<string[]>([]);

  let split = $state<Split>('train');
  let multiclass = $state(false);
  let limit = $state('');
  let threads = $state(20);
  let noLabels = $state(false);

  // image filters
  let isOccluded = $state('');
  let isTruncated = $state('');
  let isGroupOf = $state('');
  let isDepiction = $state('');
  let isInside = $state('');

  let loadingClasses = $state(false);
  let fetchingMeta = $state(false);
  let submitting = $state(false);
  let error = $state('');
  let successMsg = $state('');

  const filteredClasses = $derived(
    classSearch.trim().length < 2
      ? []
      : allClasses
          .filter((c) =>
            c.name.toLowerCase().includes(classSearch.toLowerCase())
          )
          .slice(0, 50)
  );

  // ---- lifecycle ----
  onMount(async () => {
    loadingClasses = true;
    try {
      allClasses = await invoke<ClassEntry[]>('list_classes');
    } catch (e: any) {
      error = String(e);
    } finally {
      loadingClasses = false;
    }
  });

  // ---- handlers ----
  function toggleClass(name: string) {
    if (selectedClasses.includes(name)) {
      selectedClasses = selectedClasses.filter((c) => c !== name);
    } else {
      selectedClasses = [...selectedClasses, name];
    }
    classSearch = '';
  }

  function removeSelected(name: string) {
    selectedClasses = selectedClasses.filter((c) => c !== name);
  }

  function parseBool(v: string): boolean | undefined {
    if (v === '1') return true;
    if (v === '0') return false;
    return undefined;
  }

  async function fetchMeta() {
    fetchingMeta = true;
    error = '';
    try {
      await invoke('ensure_annotation_csvs', { split });
      successMsg = `Annotation CSV(s) for "${split}" downloaded/verified.`;
    } catch (e: any) {
      error = String(e);
    } finally {
      fetchingMeta = false;
    }
  }

  async function submitDownload() {
    if (selectedClasses.length === 0) {
      error = 'Please select at least one class.';
      return;
    }
    error = '';
    successMsg = '';
    submitting = true;

    const config: DownloadConfig = {
      classes: selectedClasses,
      split,
      multiclass,
      limit: limit ? parseInt(limit) : undefined,
      threads,
      no_labels: noLabels,
      filters: {
        is_occluded: parseBool(isOccluded),
        is_truncated: parseBool(isTruncated),
        is_group_of: parseBool(isGroupOf),
        is_depiction: parseBool(isDepiction),
        is_inside: parseBool(isInside),
      },
      yes: true,
    };

    try {
      const jobId = await invoke<string>('start_download', { config });
      jobStore.addJob(jobId, selectedClasses, split);
      goto('/jobs');
    } catch (e: any) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="page">
  <h1>Download Dataset</h1>
  <p class="subtitle">
    Configure a download job. Classes and annotation CSVs are fetched from
    Open Images. Images are downloaded from the public S3 bucket over HTTPS.
  </p>

  {#if error}
    <div class="alert error">{error}</div>
  {/if}
  {#if successMsg}
    <div class="alert success">{successMsg}</div>
  {/if}

  <div class="form-grid">
    <!-- ---- Left column ---- -->
    <section class="section">
      <h2>1 · Select Classes</h2>

      {#if loadingClasses}
        <p class="muted">Loading class list from Open Images…</p>
      {:else}
        <div class="search-wrap">
          <input
            type="text"
            placeholder="Search classes (e.g. Cat, Car, Person)"
            bind:value={classSearch}
            class="input"
          />
        </div>

        {#if filteredClasses.length > 0}
          <ul class="suggestions">
            {#each filteredClasses as cls}
              <li>
                <button
                  class="suggestion-btn"
                  class:selected={selectedClasses.includes(cls.name)}
                  onclick={() => toggleClass(cls.name)}
                >
                  {cls.name}
                  <span class="code">{cls.code}</span>
                </button>
              </li>
            {/each}
          </ul>
        {:else if classSearch.length >= 2}
          <p class="muted">No matches.</p>
        {/if}

        {#if selectedClasses.length > 0}
          <div class="selected-pills">
            {#each selectedClasses as cls}
              <span class="pill">
                {cls}
                <button class="pill-remove" onclick={() => removeSelected(cls)}>×</button>
              </span>
            {/each}
          </div>
        {/if}
      {/if}
    </section>

    <!-- ---- Right column ---- -->
    <section class="section">
      <h2>2 · Dataset Options</h2>

      <label class="field">
        <span>Split</span>
        <select bind:value={split} class="input">
          <option value="train">Train</option>
          <option value="validation">Validation</option>
          <option value="test">Test</option>
          <option value="all">All</option>
        </select>
      </label>

      <label class="field">
        <span>Image limit <small>(blank = no limit)</small></span>
        <input
          type="number"
          min="1"
          placeholder="e.g. 100"
          bind:value={limit}
          class="input"
        />
      </label>

      <label class="field">
        <span>Download threads</span>
        <input
          type="number"
          min="1"
          max="50"
          bind:value={threads}
          class="input"
        />
      </label>

      <label class="checkbox">
        <input type="checkbox" bind:checked={multiclass} />
        Download all selected classes into a single merged folder
      </label>

      <label class="checkbox">
        <input type="checkbox" bind:checked={noLabels} />
        Skip YOLO label file generation
      </label>

      <h3 class="filter-heading">Image Attribute Filters <small>(blank = ignore)</small></h3>

      {#each [
        { label: 'IsOccluded', bind: isOccluded, setter: (v: string) => { isOccluded = v; } },
        { label: 'IsTruncated', bind: isTruncated, setter: (v: string) => { isTruncated = v; } },
        { label: 'IsGroupOf', bind: isGroupOf, setter: (v: string) => { isGroupOf = v; } },
        { label: 'IsDepiction', bind: isDepiction, setter: (v: string) => { isDepiction = v; } },
        { label: 'IsInside', bind: isInside, setter: (v: string) => { isInside = v; } },
      ] as f}
        <label class="field filter-row">
          <span>{f.label}</span>
          <select
            value={f.bind}
            onchange={(e) => f.setter((e.target as HTMLSelectElement).value)}
            class="input input-sm"
          >
            <option value="">—</option>
            <option value="1">Yes (1)</option>
            <option value="0">No (0)</option>
          </select>
        </label>
      {/each}
    </section>
  </div>

  <div class="actions">
    <button class="btn secondary" disabled={fetchingMeta} onclick={fetchMeta}>
      {fetchingMeta ? 'Fetching…' : '📥 Fetch Metadata CSVs'}
    </button>
    <button
      class="btn primary"
      disabled={submitting || selectedClasses.length === 0}
      onclick={submitDownload}
    >
      {submitting ? 'Starting…' : '🚀 Start Download'}
    </button>
  </div>
</div>

<style>
  .page { max-width: 1000px; }

  h1 { font-size: 1.7rem; font-weight: 800; color: #90cdf4; margin-bottom: 6px; }
  .subtitle { color: #718096; font-size: 0.88rem; margin-bottom: 24px; }

  .alert {
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 0.85rem;
    margin-bottom: 16px;
  }
  .error { background: #2d1b1b; color: #fc8181; border: 1px solid #742a2a; }
  .success { background: #1a2d1e; color: #68d391; border: 1px solid #276749; }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
    margin-bottom: 24px;
  }

  .section {
    background: #161b27;
    border: 1px solid #2d3748;
    border-radius: 12px;
    padding: 20px;
  }

  h2 { font-size: 0.95rem; font-weight: 700; color: #90cdf4; margin-bottom: 14px; }
  h3 { font-size: 0.82rem; font-weight: 600; color: #718096; margin: 14px 0 8px; }
  .filter-heading { margin-top: 18px; }

  .muted { color: #4a5568; font-size: 0.85rem; }

  .search-wrap { margin-bottom: 8px; }
  .input {
    width: 100%;
    background: #0f1117;
    border: 1px solid #2d3748;
    border-radius: 6px;
    color: #e2e8f0;
    padding: 7px 10px;
    font-size: 0.85rem;
    outline: none;
  }
  .input:focus { border-color: #4a90d9; }
  .input-sm { width: auto; }

  .suggestions {
    list-style: none;
    background: #0f1117;
    border: 1px solid #2d3748;
    border-radius: 8px;
    max-height: 200px;
    overflow-y: auto;
    margin-bottom: 10px;
  }

  .suggestion-btn {
    width: 100%;
    background: none;
    border: none;
    color: #a0aec0;
    padding: 7px 12px;
    text-align: left;
    cursor: pointer;
    font-size: 0.83rem;
    display: flex;
    justify-content: space-between;
  }
  .suggestion-btn:hover { background: #1a2535; color: #e2e8f0; }
  .suggestion-btn.selected { color: #68d391; }

  .code { color: #4a5568; font-size: 0.75rem; }

  .selected-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 10px;
  }

  .pill {
    background: #2b4b7e;
    color: #90cdf4;
    border-radius: 99px;
    padding: 3px 10px 3px 12px;
    font-size: 0.78rem;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .pill-remove {
    background: none;
    border: none;
    color: #63b3ed;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    padding: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
    font-size: 0.82rem;
    color: #a0aec0;
  }

  .filter-row {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
  .filter-row span { min-width: 100px; }

  .checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.83rem;
    color: #a0aec0;
    margin-bottom: 10px;
    cursor: pointer;
  }

  .actions {
    display: flex;
    gap: 12px;
  }

  .btn {
    padding: 10px 22px;
    border-radius: 8px;
    border: none;
    font-size: 0.88rem;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s;
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.primary { background: #2b6cb0; color: #fff; }
  .btn.primary:hover:not(:disabled) { background: #3182ce; }
  .btn.secondary { background: #2d3748; color: #e2e8f0; }
  .btn.secondary:hover:not(:disabled) { background: #4a5568; }
</style>
