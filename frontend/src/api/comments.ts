import { api } from './client'
import type { Comment } from '../types'

export async function listApproved(slug: string) {
  const { data } = await api.get<Comment[]>(`/posts/${encodeURIComponent(slug)}/comments`)
  return data
}

export interface NewCommentInput {
  author_name: string
  author_email?: string
  content: string
  parent_id?: number | null
  /// Honeypot field. Must be empty on real submissions.
  website?: string
}

export async function submit(slug: string, input: NewCommentInput) {
  const { data } = await api.post<Comment>(`/posts/${encodeURIComponent(slug)}/comments`, input)
  return data
}
