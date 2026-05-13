# Personal Blog (Vue 3 + Rust + Axum)

A self-hosted personal blog with a public reader site, a dedicated admin SPA, and a single Rust backend.

- **Backend** — Rust + Axum + SQLx + PostgreSQL 16
- **Frontends** — Vue 3 + TypeScript + Vite + Naive UI + Pinia + Vue Router (two separate SPAs: public reader + admin console)
- **Auth** — JWT Bearer (argon2 password hashing); single-author admin
- **Content** — Markdown authored in admin (md-editor-v3), backend renders to HTML on save (comrak + GFM)
- **i18n** — Chinese / English on both frontends (vue-i18n)

## Feature checklist

### Reading

- Tagged posts, paginated home, tag pages, full-text search (PostgreSQL `tsvector`)
- Markdown rendering with GFM (tables, task lists, strikethrough, footnotes, autolinks)
- Mermaid diagrams in posts
- Per-post **view counter** with per-IP 30-minute dedupe (no F5-farming)
- **Reading time** + word count (CJK-aware)
- **Table of contents** sidebar (sticky, viewport-fixed, hidden on narrow screens)
- **Related posts** (shared-tag scored) + **prev/next** navigation
- Threaded comments with moderation queue
- **SEO**: dynamic `<title>` / `og:*` / `twitter:*` per page
- **RSS feed** at `/rss.xml`, **sitemap** at `/sitemap.xml`

### Writing & moderation (admin)

- Markdown editor with live preview, syntax highlighting, Mermaid
- **Image upload** (multipart, image/\* only, UUID-renamed) — drag/paste/toolbar in the editor
- Posts: draft / publish, tags, slug, excerpt (auto if blank)
- Comment moderation: pending queue, approve / spam / delete
- Tag management, dashboard with charts, raw data tables (posts/comments/tags/users)
- **Dictionary** system for enum-style options (post status, comment status, …) editable from the UI
- **Audit log** for every write
- User management (create / reset password / delete)

### Anti-spam

- **Honeypot** field on the public comment form (hidden via CSS+aria, server discards anything filled)
- **Per-IP rate limit** (30 s between comments)
- **Keyword blocklist** (`COMMENT_BLOCKLIST` env var) — matches route the comment to `status='spam'` instead of `pending`

## Project layout

```
Ai Project/
├── backend/                Rust crate (Axum API + uploads + SPA serving)
│   ├── migrations/         sqlx migrations (auto-run on startup)
│   ├── src/
│   ├── uploads/            created on startup; image uploads land here
│   └── .env.example
├── frontend/               Vite Vue 3 TS — public reader (port 5178 in dev)
├── admin-frontend/         Vite Vue 3 TS — admin console (port 5180 in dev)
├── docker-compose.yml      PostgreSQL 16 for local dev
└── README.md
```

## Prerequisites

- Rust 1.85+ (edition 2024 — verified on 1.95)
- Node 20+ and npm
- Docker Desktop or a standalone PostgreSQL 16 instance

## First-time setup

### 1. Start Postgres

```bash
docker compose up -d
```

`postgres:16-alpine` on `localhost:5432` with `blog/blog/blog`. Data persists in the `blog_pg_data` volume.

### 2. Configure the backend

```bash
cd backend
cp .env.example .env       # edit JWT_SECRET, CORS, SITE_URL for your setup
```

Key `.env` settings:

| Variable | Default | Purpose |
|---|---|---|
| `BIND_ADDR` | `0.0.0.0:8080` | API bind address |
| `DATABASE_URL` | _(required)_ | Postgres URL |
| `JWT_SECRET` | _(required, ≥32 bytes)_ | Token signing key |
| `CORS_ALLOWED_ORIGINS` | `http://localhost:5173` | Comma-separated origins |
| `STATIC_DIR` | _(empty)_ | Set to a built SPA path for single-binary deploy |
| `SITE_URL` | `http://localhost:5173` | Absolute origin used in RSS / sitemap |
| `SITE_TITLE` | `Blog` | RSS channel title |
| `SITE_DESCRIPTION` | `Personal blog` | RSS channel description |
| `UPLOAD_DIR` | `./uploads` | Where uploaded images land (auto-created) |
| `MAX_UPLOAD_BYTES` | `5242880` | Per-upload cap (5 MB) |
| `COMMENT_BLOCKLIST` | _(empty)_ | Comma-separated lowercased substrings; matches → `status='spam'` |
| `RUST_LOG` | `info` | Tracing filter |

### 3. Run the backend

```bash
cargo run
```

Migrations apply automatically. Health: `curl http://localhost:8080/api/health` → `ok`.

### 4. Seed an admin user

```bash
cargo run --bin seed_admin -- admin
```

Prompts twice for a password (min 8 chars). Idempotent — re-running resets the password.

### 5. (optional) Seed sample posts

```bash
cargo run --bin seed_posts            # 30 posts
cargo run --bin seed_posts -- 60      # custom count
```

Templates rotate across 10 mixed CN/EN topics with realistic markdown bodies. Every slug is prefixed `seed-`, so:

```sql
DELETE FROM posts WHERE slug LIKE 'seed-%';
```

cleans them up.

### 6. Run the public frontend

```bash
cd ../frontend
npm install
npm run dev          # http://localhost:5178
```

Vite proxies `/api` and `/uploads` to the backend.

### 7. Run the admin frontend

```bash
cd ../admin-frontend
npm install
npm run dev          # http://localhost:5180
```

Login at `http://localhost:5180/login` using the credentials from step 4.

## API surface

### Public (no auth)

| Method | Path | Notes |
|---|---|---|
| `GET`  | `/api/posts?page=&per_page=&tag=` | Paginated, published only |
| `GET`  | `/api/posts/:slug` | Single post; bumps `views` (with IP dedupe) |
| `GET`  | `/api/posts/:slug/related` | `{prev, next, related}` |
| `GET`  | `/api/posts/:slug/comments` | Approved comments |
| `POST` | `/api/posts/:slug/comments` | Submit (lands in `pending` or `spam`) |
| `GET`  | `/api/tags` | All tags with `post_count` |
| `GET`  | `/api/tags/:slug` | Single tag |
| `GET`  | `/api/dict/:type_code` | Public dict items for a type |
| `GET`  | `/api/search?q=&page=` | Full-text search |
| `POST` | `/api/auth/login` | `{username, password}` → `{token, user}` |
| `GET`  | `/api/health` | Liveness probe |
| `GET`  | `/rss.xml` | RSS 2.0 feed (latest 20 published) |
| `GET`  | `/sitemap.xml` | Sitemap for posts + tags + home |
| `GET`  | `/uploads/:filename` | Served from `UPLOAD_DIR` |

### Admin (Bearer JWT, role=admin)

| Method | Path | Notes |
|---|---|---|
| `GET`    | `/api/admin/posts` | All posts (drafts + published) |
| `POST`   | `/api/admin/posts` | Create — renders Markdown → HTML, computes word_count/reading_time |
| `GET`    | `/api/admin/posts/:id` | Edit fetch |
| `PUT`    | `/api/admin/posts/:id` | Update — re-renders, re-computes metrics |
| `DELETE` | `/api/admin/posts/:id` | Delete |
| `POST`   | `/api/admin/uploads` | Multipart image upload — returns `{url, filename, size}` |
| `GET`    | `/api/admin/comments?status=…` | Moderation list |
| `PATCH`  | `/api/admin/comments/:id` | `{status}` |
| `DELETE` | `/api/admin/comments/:id` | Delete |
| `GET`/`POST`/`PUT`/`DELETE` | `/api/admin/tags*` | Tag CRUD |
| `GET`/`POST`/`PUT`/`DELETE` | `/api/admin/dict/*` | Dict type + item CRUD |
| `GET`    | `/api/admin/stats/overview` | Counts of posts/comments/tags/users |
| `GET`    | `/api/admin/stats/trend` | Posts/comments by date |
| `GET`    | `/api/admin/stats/dashboard` | Overview + top posts + recent comments + tag cloud |
| `GET`    | `/api/admin/users` | List users |
| `POST`   | `/api/admin/users` | Create |
| `PATCH`  | `/api/admin/users/:id/password` | Reset password |
| `DELETE` | `/api/admin/users/:id` | Delete |
| `GET`    | `/api/admin/data/*` | Raw paginated rows (posts/comments/tags/users) |
| `GET`    | `/api/admin/audit` | Audit log |

## Production deploy

### Single binary + reverse proxy (recommended)

```bash
cd frontend && npm run build         # → frontend/dist
cd ../admin-frontend && npm run build  # → admin-frontend/dist

cd ../backend
# Edit .env: STATIC_DIR=../frontend/dist (or wherever you host the public SPA)
cargo build --release
./target/release/blog-backend
```

The backend serves:
- `/api/*` — JSON API
- `/uploads/*` — image files from `UPLOAD_DIR`
- `/rss.xml`, `/sitemap.xml`
- everything else → `STATIC_DIR/index.html` (SPA fallback for the public site)

Put nginx / Caddy in front for TLS and to route the admin SPA. Typical layout:

```
yourdomain.com/        →  backend (serves public SPA + /api + /uploads + /rss.xml + /sitemap.xml)
admin.yourdomain.com/  →  static files from admin-frontend/dist
                          /api and /uploads proxied to backend
```

### Backups

`UPLOAD_DIR` is just a folder — back it up alongside `pg_dump`. If you run multiple replicas, point `UPLOAD_DIR` at a shared volume or S3-mounted FUSE.

## Chinese (CJK) full-text search

Default migration uses Postgres `simple` text-search config (whitespace tokenizer). It **does not tokenize Chinese / Japanese / Korean** — multi-character CJK queries get treated as a single token and rarely match.

To enable proper CJK search:

1. Install [`pg_jieba`](https://github.com/jaiminpan/pg_jieba) into your Postgres instance.
2. Add a new migration that rebuilds `posts.search_vector` with `'jiebacfg'` in place of `'simple'`.
3. Update `backend/src/handlers/search.rs` to pass `'jiebacfg'` to `websearch_to_tsquery`.

The simple-config fallback already works fine for Latin-script content and short ASCII keywords inside CJK text (e.g. searching for "Rust" or "Vue").

## Migrations

`sqlx::migrate!()` runs every `.sql` in `backend/migrations/` on startup, tracked in `_sqlx_migrations`:

| File | What it adds |
|---|---|
| `0001_init.sql` | Core schema: users, posts (with `tsvector`), tags, post_tags, comments |
| `0002_audit_log.sql` | Audit log table + admin action seeding |
| `0003_dictionary.sql` | Dict types + items (post status, comment status, etc.) |
| `0004_seed_more_audit_actions.sql` | More audit action labels |
| `0005_post_views.sql` | `posts.views` counter |
| `0006_post_reading_metrics.sql` | `posts.word_count` + `reading_time_min` (with backfill) |

## What's NOT included (intentional)

- **Multi-user roles** — a single `admin` is enough for a personal blog; the schema has room to grow if you need it.
- **Email notifications** for new comments — straightforward to bolt on (SMTP client + new-comment hook) but not done.
- **Draft preview links** — drafts are admin-only; sharing a draft for review needs a token-gated public route.
- **CSRF middleware** — not needed because auth is Bearer JWT, not cookies.
- **2FA on admin login.**
- **Per-author bylines** — every post is implicitly authored by the single admin.
