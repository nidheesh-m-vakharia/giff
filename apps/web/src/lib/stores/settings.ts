// apps/web/src/lib/stores/settings.ts
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface Settings {
  token: string;
  repo: string; // "owner/repo"
}

function createSettingsStore() {
  const initial: Settings = browser
    ? {
        token: localStorage.getItem('giff_token') ?? '',
        repo: localStorage.getItem('giff_repo') ?? ''
      }
    : { token: '', repo: '' };

  const { subscribe, set, update } = writable<Settings>(initial);

  return {
    subscribe,
    save(values: Settings) {
      if (browser) {
        localStorage.setItem('giff_token', values.token);
        localStorage.setItem('giff_repo', values.repo);
      }
      set(values);
    },
    clear() {
      if (browser) {
        localStorage.removeItem('giff_token');
        localStorage.removeItem('giff_repo');
      }
      set({ token: '', repo: '' });
    }
  };
}

export const settings = createSettingsStore();
