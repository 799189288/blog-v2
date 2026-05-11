import { api } from './client'
import type { Paginated, PostDetail, PostSummary } from '../types'

export async function listPublished(params: { page?: number; per_page?: number; tag?: string } = {}) {
  const { data } = await api.get<Paginated<PostSummary>>('/posts', { params })
  return data
}

export async function getBySlug(slug: string) {
  const { data } = await api.get<PostDetail>(`/posts/${encodeURIComponent(slug)}`)
  return data
}

export async function search(q: string, page = 1) {
  const { data } = await api.get<Paginated<PostSummary>>('/search', { params: { q, page } })
  return data
}
