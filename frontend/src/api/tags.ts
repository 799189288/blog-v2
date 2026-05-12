import { api } from './client'
import type { Tag, TagWithCount } from '../types'

export async function list() {
  const { data } = await api.get<TagWithCount[]>('/tags')
  return data
}

export async function getBySlug(slug: string) {
  const { data } = await api.get<Tag>(`/tags/${encodeURIComponent(slug)}`)
  return data
}
