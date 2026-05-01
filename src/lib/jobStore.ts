import { writable, get } from 'svelte/store';
import type { DownloadJob, ProgressEvent, Split } from './types';

function createJobStore() {
  const { subscribe, update } = writable<Map<string, DownloadJob>>(new Map());

  function addJob(jobId: string, classes: string[], split: Split) {
    update((jobs) => {
      for (const cls of classes) {
        const key = `${jobId}-${cls}-${split}`;
        jobs.set(key, {
          job_id: jobId,
          class_name: cls,
          split,
          downloaded: 0,
          total: 0,
          finished: false,
          errors: [],
          started_at: Date.now(),
        });
      }
      return jobs;
    });
  }

  function applyProgress(event: ProgressEvent) {
    update((jobs) => {
      const key = `${event.job_id}-${event.class_name}-${event.split}`;
      const existing = jobs.get(key) ?? {
        job_id: event.job_id,
        class_name: event.class_name,
        split: event.split,
        downloaded: 0,
        total: event.total,
        finished: false,
        errors: [],
        started_at: Date.now(),
      };
      existing.downloaded = event.downloaded;
      existing.total = event.total;
      existing.finished = event.finished;
      if (event.error) {
        existing.errors = [...existing.errors, event.error];
      }
      jobs.set(key, existing);
      return jobs;
    });
  }

  function clear() {
    update(() => new Map());
  }

  return { subscribe, addJob, applyProgress, clear };
}

export const jobStore = createJobStore();
