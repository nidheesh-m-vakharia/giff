// Relative-time formatter that matches GitHub's style (`5 minutes ago`, `3 days ago`, etc.).
// Dates older than ~30 days fall back to absolute "Mon DD, YYYY" so the timeline doesn't read
// "8 months ago" when the actual date is more useful.

const SECOND = 1;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;
const MONTH = 30 * DAY;

export function timeAgo(at: string | null | undefined): string {
  if (!at) return '';
  const t = new Date(at).getTime();
  if (Number.isNaN(t)) return '';
  const sec = Math.max(0, Math.floor((Date.now() - t) / 1000));

  if (sec < MINUTE) return 'just now';
  if (sec < HOUR) return plural(Math.floor(sec / MINUTE), 'minute');
  if (sec < DAY) return plural(Math.floor(sec / HOUR), 'hour');
  if (sec < WEEK) return plural(Math.floor(sec / DAY), 'day');
  if (sec < MONTH) return plural(Math.floor(sec / WEEK), 'week');
  // Past a month, absolute date is more useful than "5 months ago".
  return new Date(at).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  });
}

function plural(n: number, unit: string): string {
  return `${n} ${unit}${n === 1 ? '' : 's'} ago`;
}
