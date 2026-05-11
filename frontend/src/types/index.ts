export interface Tag {
  id: number
  name: string
  slug: string
}

export interface TagWithCount extends Tag {
  post_count: number
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

export interface Paginated<T> {
  items: T[]
  page: number
  per_page: number
  total: number
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

export interface AuthUser {
  id: number
  username: string
  role: string
}

export interface LoginResponse {
  token: string
  user: AuthUser
}
