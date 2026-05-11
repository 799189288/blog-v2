import { api } from './client'
import type { UserRow } from '../types'

export async function list() {
  const { data } = await api.get<UserRow[]>('/admin/users')
  return data
}

export async function create(username: string, password: string) {
  const { data } = await api.post<UserRow>('/admin/users', { username, password })
  return data
}

export async function resetPassword(id: number, password: string) {
  const { data } = await api.patch<UserRow>(`/admin/users/${id}/password`, { password })
  return data
}

export async function remove(id: number) {
  await api.delete(`/admin/users/${id}`)
}
