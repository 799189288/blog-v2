import { api } from './client'
import type { Tag, TagBrowseRow } from '../types'

export interface UpsertTagInput {
  name: string
  slug?: string
}

export async function adminList() {
  const { data } = await api.get<TagBrowseRow[]>('/admin/tags')
  return data
}

export async function adminCreate(input: UpsertTagInput) {
  const { data } = await api.post<Tag>('/admin/tags', input)
  return data
}

export async function adminUpdate(id: number, input: UpsertTagInput) {
  const { data } = await api.put<Tag>(`/admin/tags/${id}`, input)
  return data
}

export async function adminDelete(id: number) {
  await api.delete(`/admin/tags/${id}`)
}
