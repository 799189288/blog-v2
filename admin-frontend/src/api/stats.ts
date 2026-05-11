import { api } from './client'
import type { Overview, Trend } from '../types'

export async function overview() {
  const { data } = await api.get<Overview>('/admin/stats/overview')
  return data
}

export async function trend(days: number) {
  const { data } = await api.get<Trend>('/admin/stats/trend', { params: { days } })
  return data
}
