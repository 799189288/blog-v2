import { api } from './client'
import type { Overview, Trend, DashboardResponse } from '../types'

export async function overview() {
  const { data } = await api.get<Overview>('/admin/stats/overview')
  return data
}

export async function trend(days: number) {
  const { data } = await api.get<Trend>('/admin/stats/trend', { params: { days } })
  return data
}

export async function dashboard() {
  const { data } = await api.get<DashboardResponse>('/admin/stats/dashboard')
  return data
}
