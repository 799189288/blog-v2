import { api } from './client'
import type { AuditRow, Page } from '../types'

export interface AuditParams {
  page?: number
  per_page?: number
  user_id?: number
  action?: string
  from?: string
  to?: string
}

export async function list(params: AuditParams = {}) {
  const { data } = await api.get<Page<AuditRow>>('/admin/audit', { params })
  return data
}
