// Minimal unified-diff parser for the `patch` strings GitHub returns from /pulls/:n/files.
// Output: an ordered list of hunks per file, each hunk a list of lines tagged context/add/del.

export type LineKind = 'context' | 'add' | 'del' | 'meta';

export interface DiffLine {
  kind: LineKind;
  text: string;
  oldNum: number | null;
  newNum: number | null;
}

export interface Hunk {
  header: string;
  lines: DiffLine[];
}

const HUNK_RE = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/;

export function parsePatch(patch: string | undefined): Hunk[] {
  if (!patch) return [];
  const out: Hunk[] = [];
  let current: Hunk | null = null;
  let oldNum = 0;
  let newNum = 0;

  for (const line of patch.split('\n')) {
    const m = line.match(HUNK_RE);
    if (m) {
      if (current) out.push(current);
      oldNum = parseInt(m[1], 10);
      newNum = parseInt(m[2], 10);
      current = { header: line, lines: [] };
      continue;
    }
    if (!current) continue;

    if (line.startsWith('+')) {
      current.lines.push({ kind: 'add', text: line.slice(1), oldNum: null, newNum });
      newNum++;
    } else if (line.startsWith('-')) {
      current.lines.push({ kind: 'del', text: line.slice(1), oldNum, newNum: null });
      oldNum++;
    } else if (line.startsWith('\\')) {
      current.lines.push({ kind: 'meta', text: line, oldNum: null, newNum: null });
    } else {
      const text = line.startsWith(' ') ? line.slice(1) : line;
      current.lines.push({ kind: 'context', text, oldNum, newNum });
      oldNum++;
      newNum++;
    }
  }
  if (current) out.push(current);
  return out;
}

// Best-guess shiki language id from filename extension.
export function shikiLang(filename: string): string {
  const lower = filename.toLowerCase();
  const dot = lower.lastIndexOf('.');
  if (dot < 0) return 'text';
  const ext = lower.slice(dot + 1);
  const map: Record<string, string> = {
    ts: 'typescript',
    tsx: 'tsx',
    js: 'javascript',
    jsx: 'jsx',
    rs: 'rust',
    py: 'python',
    rb: 'ruby',
    go: 'go',
    java: 'java',
    kt: 'kotlin',
    swift: 'swift',
    c: 'c',
    h: 'c',
    cpp: 'cpp',
    cc: 'cpp',
    hpp: 'cpp',
    cs: 'csharp',
    php: 'php',
    sh: 'bash',
    bash: 'bash',
    zsh: 'bash',
    md: 'markdown',
    json: 'json',
    yaml: 'yaml',
    yml: 'yaml',
    toml: 'toml',
    html: 'html',
    css: 'css',
    scss: 'scss',
    svelte: 'svelte',
    vue: 'vue',
    sql: 'sql',
    xml: 'xml'
  };
  return map[ext] ?? 'text';
}
