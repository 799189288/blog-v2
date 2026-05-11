import { api } from './client'
import type { Page, PostBrowseRow, CommentBrowseRow, TagBrowseRow, UserRow } from '../types'

export interface ListParams {
  page?: number
  per_page?: number
  sort?: string
  dir?: 'asc' | 'desc'
  q?: string
  status?: string
  post_id?: number
}

export async function listPosts(params: ListParams = {}) {
  const { data } = await api.get<Page<PostBrowseRow>>('/admin/data/posts', { params })
  return data
}

export async function listComments(params: ListParams = {}) {
  const { data } = await api.get<Page<CommentBrowseRow>>('/admin/data/comments', { params })
  return data
}

export async function listTags(params: ListParams = {}) {
  const { data } = await api.get<Page<TagBrowseRow>>('/admin/data/tags', { params })
  return data
}

export async function listUsers(params: ListParams = {}) {
  const { data } = await api.get<Page<UserRow>>('/admin/data/users', { params })
  return data
}
