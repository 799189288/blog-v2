import { api } from './client'
import type { Comment } from '../types'

export async function adminList(status?: 'pending' | 'approved' | 'spam') {
  const params = status ? { status } : undefined
  const { data } = await api.get<Comment[]>('/admin/comments', { params })
  return data
}

export async function adminSetStatus(id: number, status: 'pending' | 'approved' | 'spam') {
  const { data } = await api.patch<Comment>(`/admin/comments/${id}`, { status })
  return data
}

export async function adminDelete(id: number) {
  await api.delete(`/admin/comments/${id}`)
}
