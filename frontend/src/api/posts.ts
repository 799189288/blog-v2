import { api } from './client'
import type { Paginated, PostDetail, PostNav, PostSummary } from '../types'

export async function listPublished(params: { page?: number; per_page?: number; tag?: string } = {}) {
  const { data } = await api.get<Paginated<PostSummary>>('/posts', { params })
  return data
}

export async function getBySlug(slug: string, token?: string) {
  const { data } = await api.get<PostDetail>(`/posts/${encodeURIComponent(slug)}`, {
    params: token ? { token } : undefined,
  })
  return data
}

export async function getRelated(slug: string) {
  const { data } = await api.get<PostNav>(`/posts/${encodeURIComponent(slug)}/related`)
  return data
}

export async function search(q: string, page = 1) {
  const { data } = await api.get<Paginated<PostSummary>>('/search', { params: { q, page } })
  return data
}
