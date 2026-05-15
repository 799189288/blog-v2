// Rewrite relative /uploads/... links in user-authored markdown so they
// resolve against the backend origin instead of the public site origin.
//
// Why: the backend returns `/uploads/<uuid>.webp` after image upload, and
// the admin editor embeds that as `![](/uploads/...)` into the markdown.
// When the public site and backend share an origin (dev with Vite proxy,
// or a reverse proxy in front of both in prod) the relative URL just
// works. On split-domain deploys (e.g. Railway with separate public-site
// and backend services) the relative URL resolves to the public site,
// which has no /uploads route and returns 404.
//
// Strategy: derive the backend origin from VITE_API_BASE_URL. If that
// is itself a relative path (default '/api'), assume same-origin and
// return the markdown unchanged. Otherwise prefix every `/uploads/...`
// occurrence inside markdown link / image / raw HTML attribute syntax
// with the origin. Absolute URLs (containing '://') and links pointing
// anywhere other than `/uploads/` are left alone.

const API_BASE = (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? '/api'

function deriveOrigin(base: string): string {
  // Same-origin deployments use a relative base ('/api', or anything not
  // starting with http/https). Nothing to rewrite in that case.
  if (!/^https?:\/\//i.test(base)) return ''
  try {
    return new URL(base).origin
  } catch {
    return ''
  }
}

const ORIGIN = deriveOrigin(API_BASE)

export function rewriteUploads(md: string): string {
  if (!ORIGIN || !md) return md
  // Match markdown link/image targets `](/uploads/...)`, raw HTML
  // attribute values `src="/uploads/..."` / `href="/uploads/..."`, and
  // bare `(/uploads/...)`. Capture the path tail so we can re-emit it
  // after the prefix.
  return md
    .replace(/(\]\()\/uploads\//g, `$1${ORIGIN}/uploads/`)
    .replace(/(src=["'])\/uploads\//g, `$1${ORIGIN}/uploads/`)
    .replace(/(href=["'])\/uploads\//g, `$1${ORIGIN}/uploads/`)
}
