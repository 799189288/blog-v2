# Personal Blog (Vue 3 + Rust + Axum)

A self-hosted personal blog with:

- **Frontend**: Vue 3 + TypeScript + Vite + Naive UI + Pinia + Vue Router
- **Backend**: Rust + Axum + SQLx + PostgreSQL
- **Auth**: JWT-based admin login (argon2 password hashing)
- **Content**: Markdown authored in admin panel, server-rendered to HTML on save (via comrak, with GFM)
- **Features**: tagged posts, reader comments with moderation, full-text search (PostgreSQL `tsvector`)

## Project layout

```
Ai Project/
├── backend/            Rust crate (Axum API + static SPA serving)
├── frontend/           Vite Vue 3 TS app
├── docker-compose.yml  PostgreSQL 16 for local dev
└── .gitignore
```

## Prerequisites

- Rust (1.85+ for edition 2024 — verified on 1.95)
- Node 20+ and npm
- Docker Desktop (for the local Postgres container) — or your own PostgreSQL 16 server

## First-time setup

### 1. Start Postgres

```bash
docker compose up -d
```

This boots `postgres:16-alpine` on `localhost:5432` with user/password/db `blog/blog/blog`. Data persists in the `blog_pg_data` volume.

### 2. Configure the backend

```bash
cd backend
cp .env.example .env       # already provided in this repo; edit if you wish
```

`.env` keys:
- `BIND_ADDR` — what the API binds to (default `0.0.0.0:8080`)
- `DATABASE_URL` — PostgreSQL connection string
- `JWT_SECRET` — **change in production** (must be ≥32 bytes)
- `CORS_ALLOWED_ORIGINS` — comma-separated origin list (default `http://localhost:5173`)
- `STATIC_DIR` — leave empty in dev; set to `../frontend/dist` for single-binary deploy
- `RUST_LOG` — tracing filter

### 3. Run the backend

```bash
cargo run                  # migrations run automatically on startup
```

Health check: `curl http://localhost:8080/api/health` → `ok`

### 4. Seed an admin user

```bash
cargo run --bin seed_admin -- admin
```

You'll be prompted twice for a password (min 8 chars). The user is upserted, so re-running the command resets the password.

### 5. Run the frontend

```bash
cd ../frontend
npm install
npm run dev
```

Open <http://localhost:5173>. Vite proxies `/api/*` to the backend on `:8080`.

Admin login lives at <http://localhost:5173/admin/login>.

## API surface

### Public (no auth)

| Method | Path | Notes |
|---|---|---|
| `GET`  | `/api/posts?page=&per_page=&tag=` | Paginated list (published only) |
| `GET`  | `/api/posts/:slug` | Single post (published only) |
| `GET`  | `/api/posts/:slug/comments` | Approved comments |
| `POST` | `/api/posts/:slug/comments` | Submit a comment (lands in `pending`) |
| `GET`  | `/api/tags` | All tags with `post_count` |
| `GET`  | `/api/search?q=&page=` | Full-text search |
| `POST` | `/api/auth/login` | `{username, password}` → `{token, user}` |
| `GET`  | `/api/health` | Liveness probe |

### Admin (Bearer JWT, role=admin)

| Method | Path | Notes |
|---|---|---|
| `GET`    | `/api/admin/posts` | All posts (drafts + published) |
| `POST`   | `/api/admin/posts` | Create — renders Markdown → HTML on save |
| `GET`    | `/api/admin/posts/:id` | Edit fetch (returns `content_md`) |
| `PUT`    | `/api/admin/posts/:id` | Update — re-renders HTML |
| `DELETE` | `/api/admin/posts/:id` | Delete |
| `GET`    | `/api/admin/comments?status=pending\|approved\|spam` | Moderation list |
| `PATCH`  | `/api/admin/comments/:id` | `{status}` |
| `DELETE` | `/api/admin/comments/:id` | Delete |

## Production deploy (single binary)

```bash
cd frontend
npm run build              # outputs to frontend/dist

cd ../backend
# Edit .env: STATIC_DIR=../frontend/dist
cargo build --release
./target/release/blog-backend
```

The backend serves `/api/*` and falls back to `dist/index.html` for any other path (SPA fallback). Put nginx/Caddy in front for TLS.

## Chinese (CJK) full-text search

The default migration uses PostgreSQL's `simple` text search config, which splits on whitespace and works for languages with word boundaries (English, etc.). It **will not tokenize Chinese, Japanese, or Korean**.

To support Chinese:

1. Install [`pg_jieba`](https://github.com/jaiminpan/pg_jieba) into your Postgres instance.
2. Replace `'simple'` with `'jiebacfg'` in `backend/migrations/0001_init.sql` *before* running migrations (or write a new migration that rebuilds the generated column).
3. The search handler in `backend/src/handlers/search.rs` should also switch the second arg of `websearch_to_tsquery` to `'jiebacfg'`.

Alternative (no extension): switch the search to `pg_trgm` + ILIKE similarity. Slower for large corpora but works for any language.

## What's NOT included (intentional)

- Multi-user / role hierarchy — a single `admin` role is enough for a personal blog
- RSS/Atom feed — one handler away; easy to add later
- Image uploads — paste image URLs into Markdown; add `/api/admin/uploads` if needed
- Email notifications for new comments
- CSRF middleware — not needed because auth is Bearer JWT, not cookies
- i18n
