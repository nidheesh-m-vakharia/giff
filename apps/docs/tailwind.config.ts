import type { Config } from 'tailwindcss';
import defaultTheme from 'tailwindcss/defaultTheme';
import typography from '@tailwindcss/typography';

export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Geist Variable"', ...defaultTheme.fontFamily.sans],
        mono: ['"Geist Mono Variable"', ...defaultTheme.fontFamily.mono]
      },
      colors: {
        brand: {
          DEFAULT: '#ff0035',
          fg: '#ffffff'
        },
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        muted: {
          DEFAULT: 'hsl(var(--muted))',
          foreground: 'hsl(var(--muted-foreground))'
        },
        accent: {
          DEFAULT: 'hsl(var(--accent))',
          foreground: 'hsl(var(--accent-foreground))'
        },
        card: {
          DEFAULT: 'hsl(var(--card))',
          foreground: 'hsl(var(--card-foreground))'
        }
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)'
      },
      typography: ({ theme }: { theme: (path: string) => string }) => ({
        // Hierarchy here is communicated by COLOR and WEIGHT, not size.
        // Only h1 (page title) gets a size bump. h2/h3/p are all body-sized; the eye reads
        // depth from foreground → muted-foreground and from semibold → medium → regular.
        // Code blocks are LIGHT (bg-muted, dark text), not the dark editor look.
        DEFAULT: {
          css: {
            '--tw-prose-body': theme('colors.foreground'),
            '--tw-prose-headings': theme('colors.foreground'),
            '--tw-prose-links': theme('colors.foreground'),
            '--tw-prose-bold': theme('colors.foreground'),
            '--tw-prose-counters': theme('colors.muted.foreground'),
            '--tw-prose-bullets': theme('colors.muted.foreground'),
            '--tw-prose-hr': theme('colors.border'),
            '--tw-prose-quotes': theme('colors.foreground'),
            '--tw-prose-quote-borders': theme('colors.brand.DEFAULT'),
            '--tw-prose-code': theme('colors.foreground'),
            '--tw-prose-th-borders': theme('colors.border'),
            '--tw-prose-td-borders': theme('colors.border'),
            maxWidth: '70ch',

            a: {
              textDecoration: 'underline',
              textUnderlineOffset: '3px',
              textDecorationColor: theme('colors.muted.foreground'),
              fontWeight: '400',
              '&:hover': {
                color: theme('colors.brand.DEFAULT'),
                textDecorationColor: theme('colors.brand.DEFAULT')
              }
            },

            // Inline code — same density as body, faint outline.
            code: {
              fontWeight: '500',
              fontSize: '0.875em',
              padding: '0.1rem 0.35rem',
              borderRadius: '4px',
              backgroundColor: theme('colors.muted.DEFAULT'),
              border: `1px solid ${theme('colors.border')}`,
              color: theme('colors.foreground')
            },
            'code::before': { content: '""' },
            'code::after': { content: '""' },

            // Code blocks — LIGHT theme.
            pre: {
              backgroundColor: theme('colors.muted.DEFAULT'),
              color: theme('colors.foreground'),
              border: `1px solid ${theme('colors.border')}`,
              borderRadius: theme('borderRadius.md'),
              fontSize: '0.875em',
              lineHeight: '1.7',
              padding: '1rem 1.1rem'
            },
            'pre code': {
              backgroundColor: 'transparent',
              border: 'none',
              padding: '0',
              color: 'inherit',
              fontWeight: '400'
            },

            // Headings — depth comes from color/weight, not size.
            h1: {
              fontSize: '1.875rem',
              fontWeight: '600',
              // Apple display-type signature: very tight tracking on the page title.
              letterSpacing: '-0.035em',
              marginTop: '0',
              marginBottom: '0.6em'
            },
            h2: {
              fontSize: '1em',
              fontWeight: '600',
              color: theme('colors.foreground'),
              marginTop: '2.4em',
              marginBottom: '0.4em',
              borderBottom: 'none',
              paddingBottom: '0'
            },
            h3: {
              fontSize: '1em',
              fontWeight: '500',
              color: theme('colors.muted.foreground'),
              marginTop: '1.6em',
              marginBottom: '0.3em'
            },
            'h2 + p, h2 + ul, h2 + ol': { marginTop: '0.5em' },
            'h3 + p, h3 + ul, h3 + ol': { marginTop: '0.4em' },

            // Lead paragraph (intro graph) — pull back to muted to set up the body.
            '.lead': {
              color: theme('colors.muted.foreground'),
              fontSize: '1em'
            },

            'ul > li::marker': { color: theme('colors.muted.foreground') },
            'ol > li::marker': { color: theme('colors.muted.foreground') }
          }
        }
      })
    }
  },
  plugins: [typography]
} satisfies Config;
