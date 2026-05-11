import { api } from './client'
import type { TagWithCount } from '../types'

export async function list() {
  const { data } = await api.get<TagWithCount[]>('/tags')
  return data
}
