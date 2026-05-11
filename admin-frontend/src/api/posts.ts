import { api } from './client'
import type { PostDetail, PostInput, PostSummary } from '../types'

export async function adminList() {
  const { data } = await api.get<{ items: PostSummary[] }>('/admin/posts')
  return data.items
}

export async function adminGet(id: number) {
  const { data } = await api.get<PostDetail>(`/admin/posts/${id}`)
  return data
}

export async function adminCreate(input: PostInput) {
  const { data } = await api.post<PostDetail>('/admin/posts', input)
  return data
}

export async function adminUpdate(id: number, input: Partial<PostInput>) {
  const { data } = await api.put<PostDetail>(`/admin/posts/${id}`, input)
  return data
}

export async function adminDelete(id: number) {
  await api.delete(`/admin/posts/${id}`)
}
