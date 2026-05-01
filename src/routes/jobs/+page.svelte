<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import { jobStore } from '$lib/jobStore';
  import type { ProgressEvent, DownloadJob } from '$lib/types';

  let jobs = $state<DownloadJob[]>([]);
  let unlisten: (() => void) | undefined;

  // Sync from store
  const unsub = jobStore.subscribe((map) => {
    jobs = Array.from(map.values()).sort((a, b) => b.started_at - a.started_at);
  });

  onMount(async () => {
    unlisten = await listen<ProgressEvent>('download-progress', (event) => {
      jobStore.applyProgress(event.payload);
    });
  });

  onDestroy(() => {
    unlisten?.();
    unsub();
  });

  function percent(job: DownloadJob): number {
    if (job.total === 0) return 0;
    return Math.round((job.downloaded / job.total) * 100);
  }

  function elapsed(job: DownloadJob): string {
    const secs = Math.round((Date.now() - job.started_at) / 1000);
    if (secs < 60) return `${secs}s`;
    return `${Math.floor(secs / 60)}m ${secs % 60}s`;
  }
</script>

<div class="page">
  <h1>Download Jobs</h1>
  <p class="subtitle">Live progress for active and completed download jobs.</p>

  {#if jobs.length === 0}
    <div class="empty">
      <span class="empty-icon">📭</span>
      <p>No jobs yet. Go to <a href="/download">Download</a> to start one.</p>
    </div>
  {:else}
    <div class="jobs-list">
      {#each jobs as job (job.job_id + job.class_name + job.split)}
        <div class="job-card" class:done={job.finished} class:has-errors={job.errors.length > 0}>
          <div class="job-header">
            <span class="job-class">{job.class_name}</span>
            <span class="job-split badge">{job.split}</span>
            {#if job.finished}
              <span class="badge done-badge">✅ Done</span>
            {:else}
              <span class="badge running-badge">⏳ Running</span>
            {/if}
            <span class="elapsed">{elapsed(job)}</span>
          </div>

          <div class="progress-bar-wrap">
            <div
              class="progress-bar"
              style:width="{percent(job)}%"
              class:complete={job.finished}
            ></div>
          </div>

          <div class="job-stats">
            <span>{job.downloaded} / {job.total === 0 ? '?' : job.total} images</span>
            <span>{percent(job)}%</span>
          </div>

          {#if job.errors.length > 0}
            <details class="errors">
              <summary>{job.errors.length} error(s)</summary>
              <ul>
                {#each job.errors.slice(0, 20) as err}
                  <li>{err}</li>
                {/each}
              </ul>
            </details>
          {/if}

          <div class="job-meta">
            <code class="job-id">{job.job_id}</code>
          </div>
        </div>
      {/each}
    </div>

    <button class="btn secondary" onclick={() => jobStore.clear()}>
      Clear All
    </button>
  {/if}
</div>

<style>
  .page { max-width: 800px; }
  h1 { font-size: 1.7rem; font-weight: 800; color: #90cdf4; margin-bottom: 6px; }
  .subtitle { color: #718096; font-size: 0.88rem; margin-bottom: 24px; }

  .empty {
    text-align: center;
    padding: 60px 20px;
    color: #4a5568;
  }
  .empty-icon { font-size: 3rem; display: block; margin-bottom: 12px; }
  .empty a { color: #63b3ed; }

  .jobs-list { display: flex; flex-direction: column; gap: 14px; margin-bottom: 20px; }

  .job-card {
    background: #161b27;
    border: 1px solid #2d3748;
    border-radius: 10px;
    padding: 16px;
  }
  .job-card.done { border-color: #276749; }
  .job-card.has-errors { border-color: #742a2a; }

  .job-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }

  .job-class { font-weight: 700; color: #e2e8f0; font-size: 0.95rem; }

  .badge {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 99px;
    background: #2d3748;
    color: #a0aec0;
  }
  .job-split { background: #2b4b7e; color: #90cdf4; }
  .done-badge { background: #1a2d1e; color: #68d391; }
  .running-badge { background: #2d2b00; color: #f6e05e; }

  .elapsed { margin-left: auto; font-size: 0.75rem; color: #4a5568; }

  .progress-bar-wrap {
    height: 6px;
    background: #2d3748;
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 6px;
  }

  .progress-bar {
    height: 100%;
    background: #3182ce;
    transition: width 0.3s ease;
    border-radius: 3px;
  }
  .progress-bar.complete { background: #38a169; }

  .job-stats {
    display: flex;
    justify-content: space-between;
    font-size: 0.78rem;
    color: #718096;
    margin-bottom: 6px;
  }

  .errors {
    margin-top: 8px;
    font-size: 0.78rem;
    color: #fc8181;
  }
  .errors summary { cursor: pointer; }
  .errors ul { margin-top: 6px; padding-left: 16px; }
  .errors li { margin-bottom: 2px; }

  .job-meta { margin-top: 8px; }
  .job-id { font-size: 0.68rem; color: #4a5568; }

  .btn {
    padding: 8px 18px;
    border-radius: 8px;
    border: none;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }
  .btn.secondary { background: #2d3748; color: #e2e8f0; }
  .btn.secondary:hover { background: #4a5568; }
</style>
