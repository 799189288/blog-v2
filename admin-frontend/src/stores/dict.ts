import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import * as dictApi from '../api/dict'
import type { DictItemPublic } from '../api/dict'

/**
 * Dictionary store. One source of truth for label rendering across the app.
 *
 *   const dict = useDictStore()
 *   await dict.ensure('audit.action')      // loads + caches if not yet loaded
 *   dict.label('audit.action', 'post.create')   // -> '创建文章' / 'Create post'
 *
 * Cache is in-memory; if the admin edits a dict in the management page,
 * call `dict.reload(typeCode)` after the write to refresh.
 */
export const useDictStore = defineStore('dict', () => {
  const byType = ref<Record<string, DictItemPublic[]>>({})
  const loading = ref<Record<string, boolean>>({})
  const { locale } = useI18n()

  async function ensure(typeCode: string) {
    if (byType.value[typeCode] || loading.value[typeCode]) return
    loading.value[typeCode] = true
    try {
      const items = await dictApi.getByType(typeCode)
      byType.value[typeCode] = items
    } catch (e) {
      byType.value[typeCode] = []
      // eslint-disable-next-line no-console
      console.warn(`[dict] failed to load ${typeCode}`, e)
    } finally {
      loading.value[typeCode] = false
    }
  }

  async function ensureAll(codes: string[]) {
    await Promise.all(codes.map(ensure))
  }

  function reload(typeCode: string) {
    delete byType.value[typeCode]
    return ensure(typeCode)
  }

  function items(typeCode: string): DictItemPublic[] {
    return byType.value[typeCode] ?? []
  }

  function label(typeCode: string, code: string | null | undefined, fallback?: string): string {
    if (!code) return fallback ?? ''
    const item = (byType.value[typeCode] ?? []).find(i => i.code === code)
    if (!item) return fallback ?? code
    return locale.value === 'zh' ? item.label_zh : item.label_en
  }

  /** Reactive computed dropdown options for a type. */
  function options(typeCode: string) {
    return computed(() => items(typeCode).map(i => ({
      label: locale.value === 'zh' ? i.label_zh : i.label_en,
      value: i.code,
    })))
  }

  return { ensure, ensureAll, reload, items, label, options }
})
