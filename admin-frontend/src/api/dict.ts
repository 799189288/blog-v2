import { api } from './client'

// ---------- Types ----------

export interface DictType {
  id: number
  code: string
  name_zh: string
  name_en: string
  is_system: boolean
  created_at: string
}

export interface DictItem {
  id: number
  type_id: number
  code: string
  label_zh: string
  label_en: string
  sort: number
  enabled: boolean
  created_at: string
}

export interface UpsertDictType {
  code: string
  name_zh: string
  name_en: string
}

export interface UpsertDictItem {
  code: string
  label_zh: string
  label_en: string
  sort?: number
  enabled?: boolean
}

// ---------- Public dict (already in api/dict.ts) ----------

export interface DictItemPublic {
  code: string
  label_zh: string
  label_en: string
  sort: number
}

export async function getByType(typeCode: string) {
  const { data } = await api.get<DictItemPublic[]>(`/dict/${encodeURIComponent(typeCode)}`)
  return data
}

// ---------- Admin ----------

export async function adminListTypes() {
  const { data } = await api.get<DictType[]>('/admin/dict/types')
  return data
}

export async function adminCreateType(input: UpsertDictType) {
  const { data } = await api.post<DictType>('/admin/dict/types', input)
  return data
}

export async function adminUpdateType(id: number, input: UpsertDictType) {
  const { data } = await api.put<DictType>(`/admin/dict/types/${id}`, input)
  return data
}

export async function adminDeleteType(id: number) {
  await api.delete(`/admin/dict/types/${id}`)
}

export async function adminListItems(typeId: number) {
  const { data } = await api.get<DictItem[]>(`/admin/dict/types/${typeId}/items`)
  return data
}

export async function adminCreateItem(typeId: number, input: UpsertDictItem) {
  const { data } = await api.post<DictItem>(`/admin/dict/types/${typeId}/items`, input)
  return data
}

export async function adminUpdateItem(id: number, input: UpsertDictItem) {
  const { data } = await api.put<DictItem>(`/admin/dict/items/${id}`, input)
  return data
}

export async function adminDeleteItem(id: number) {
  await api.delete(`/admin/dict/items/${id}`)
}
