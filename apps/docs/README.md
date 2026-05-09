# giff-docs

Public docs site for giff. SvelteKit + Tailwind, deploys to Vercel.

## Local dev

```sh
npm install
npm run dev      # http://localhost:5173
```

## Build

```sh
npm run build
```

## Deploy to Vercel

Vercel detects SvelteKit automatically. Either:

- **From the dashboard:** import the repo, set the project root to `apps/docs/`, accept the
  detected SvelteKit defaults.
- **From the CLI:**
  ```sh
  cd apps/docs
  vercel deploy
  ```

The site is fully static at runtime — no env vars, no edge functions, no data fetching.
