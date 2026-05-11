export interface AuthUser {
  id: number
  username: string
  role: string
}

export interface LoginResponse {
  token: string
  user: AuthUser
}

export interface Page<T> {
  items: T[]
  page: number
  per_page: number
  total: number
}

export interface Overview {
  posts: { total: number; published: number; draft: number }
  comments: { total: number; pending: number; approved: number; spam: number }
  tags: { total: number }
  users: { total: number }
}

export interface TrendPoint {
  date: string
  count: number
}

export interface Trend {
  posts: TrendPoint[]
  comments: TrendPoint[]
}

export interface UserRow {
  id: number
  username: string
  role: string
  created_at: string
}

export interface PostBrowseRow {
  id: number
  slug: string
  title: string
  status: 'draft' | 'published'
  author_id: number
  published_at: string | null
  created_at: string
  updated_at: string
}

export interface CommentBrowseRow {
  id: number
  post_id: number
  parent_id: number | null
  author_name: string
  author_email: string | null
  content: string
  status: 'pending' | 'approved' | 'spam'
  created_at: string
}

export interface TagBrowseRow {
  id: number
  name: string
  slug: string
  post_count: number
}

export interface AuditRow {
  id: number
  user_id: number | null
  username: string
  action: string
  target_type: string | null
  target_id: number | null
  detail: unknown | null
  ip: string | null
  created_at: string
}

// Post / comment management

export interface Tag {
  id: number
  name: string
  slug: string
}

export interface PostSummary {
  id: number
  slug: string
  title: string
  excerpt: string | null
  status: 'draft' | 'published'
  published_at: string | null
  created_at: string
  tags: Tag[]
}

export interface PostDetail extends PostSummary {
  content_md: string
  content_html: string
  updated_at: string
}

export interface PostInput {
  title: string
  slug?: string
  excerpt?: string
  content_md: string
  status?: 'draft' | 'published'
  tags?: string[]
}

export interface Comment {
  id: number
  post_id: number
  parent_id: number | null
  author_name: string
  author_email: string | null
  content: string
  status: 'pending' | 'approved' | 'spam'
  created_at: string
}

