import adapter from '@sveltejs/adapter-vercel';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    // Pin the runtime so the local Node version doesn't decide the deployment runtime.
    // Vercel's actively-supported Node runtimes at time of writing are 20.x and 22.x.
    adapter: adapter({ runtime: 'nodejs20.x' })
  }
};

export default config;
