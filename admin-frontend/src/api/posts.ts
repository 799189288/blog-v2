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

export interface UploadResponse {
  url: string
  filename: string
  size: number
}

/// Uploads one image at a time via multipart and returns the public URL
/// the editor can embed. md-editor-v3 hands us a File list in its
/// onUploadImg callback; we call this in series and collect URLs.
export async function uploadImage(file: File): Promise<UploadResponse> {
  const form = new FormData()
  form.append('file', file)
  const { data } = await api.post<UploadResponse>('/admin/uploads', form, {
    headers: { 'Content-Type': 'multipart/form-data' },
    timeout: 60_000,
  })
  return data
}
