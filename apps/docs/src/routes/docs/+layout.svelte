<script lang="ts">
  import { page } from '$app/stores';
  import { cn } from '$lib/utils';

  type NavItem = { href: string; label: string };
  type NavGroup = { heading: string; items: NavItem[] };

  const groups: NavGroup[] = [
    {
      heading: 'Getting started',
      items: [
        { href: '/docs', label: 'Overview' },
        { href: '/docs/install', label: 'Installation' },
        { href: '/docs/concepts', label: 'Concepts & first principles' }
      ]
    },
    {
      heading: 'Reference',
      items: [
        { href: '/docs/commands', label: 'Commands' }
      ]
    },
    {
      heading: 'Project',
      items: [
        { href: '/docs/why', label: 'Why we made this' },
        { href: '/docs/limitations', label: 'Limitations' },
        { href: '/docs/contributing', label: 'Contributing' }
      ]
    }
  ];
</script>

<div class="mx-auto max-w-6xl px-6 py-8 flex-1">
  <div class="grid gap-12 lg:grid-cols-[14rem_minmax(0,1fr)]">
    <aside class="lg:sticky lg:top-20 lg:self-start lg:max-h-[calc(100vh-6rem)] lg:overflow-y-auto">
      <nav class="space-y-7 text-sm">
        {#each groups as group (group.heading)}
          <div class="space-y-2">
            <div class="text-xs text-muted-foreground/70 font-medium">
              {group.heading}
            </div>
            <ul class="space-y-1">
              {#each group.items as item (item.href)}
                <li>
                  <a
                    href={item.href}
                    class={cn(
                      'block py-0.5 transition-colors',
                      $page.url.pathname === item.href
                        ? 'text-foreground font-medium'
                        : 'text-muted-foreground hover:text-foreground'
                    )}
                  >
                    {item.label}
                  </a>
                </li>
              {/each}
            </ul>
          </div>
        {/each}
      </nav>
    </aside>

    <main class="min-w-0">
      <article class="prose prose-sm sm:prose-base max-w-none">
        <slot />
      </article>
    </main>
  </div>
</div>
