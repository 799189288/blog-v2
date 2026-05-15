<script setup lang="ts">
// Extracted from ConsoleLayout so the same menu can render both inside
// NLayoutSider (desktop) and inside an NDrawer (mobile hamburger).
// Emits @navigate when any leaf is clicked so the parent can close the
// drawer; on desktop the parent simply doesn't listen.

import { computed, h } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { NMenu, NIcon } from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import {
  StatsChartOutline, DocumentTextOutline, ChatbubbleOutline,
  PricetagsOutline, PeopleOutline, PersonCircleOutline, ListOutline,
  CreateOutline, BookOutline, SettingsOutline,
} from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

const emit = defineEmits<{ navigate: [] }>()
const { t } = useI18n()
const router = useRouter()

function icon(comp: any) { return () => h(NIcon, null, () => h(comp)) }

const menuOptions = computed<MenuOption[]>(() => [
  { label: () => h(RouterLink, { to: { name: 'dashboard' } }, () => t('menu.dashboard')), key: 'dashboard', icon: icon(StatsChartOutline) },
  {
    label: t('menu.content'),
    key: 'manage',
    icon: icon(CreateOutline),
    children: [
      { label: () => h(RouterLink, { to: { name: 'manage-posts' } }, () => t('menu.posts')), key: 'manage-posts', icon: icon(DocumentTextOutline) },
      { label: () => h(RouterLink, { to: { name: 'manage-post-new' } }, () => t('menu.newPost')), key: 'manage-post-new', icon: icon(CreateOutline) },
      { label: () => h(RouterLink, { to: { name: 'manage-tags' } }, () => t('menu.tags')), key: 'manage-tags', icon: icon(PricetagsOutline) },
      { label: () => h(RouterLink, { to: { name: 'manage-comments' } }, () => t('menu.commentModeration')), key: 'manage-comments', icon: icon(ChatbubbleOutline) },
    ],
  },
  {
    label: t('menu.browseData'),
    key: 'data',
    icon: icon(ListOutline),
    children: [
      { label: () => h(RouterLink, { to: { name: 'data-posts' } }, () => t('menu.dataPosts')), key: 'data-posts', icon: icon(DocumentTextOutline) },
      { label: () => h(RouterLink, { to: { name: 'data-comments' } }, () => t('menu.dataComments')), key: 'data-comments', icon: icon(ChatbubbleOutline) },
      { label: () => h(RouterLink, { to: { name: 'data-tags' } }, () => t('menu.dataTags')), key: 'data-tags', icon: icon(PricetagsOutline) },
      { label: () => h(RouterLink, { to: { name: 'data-users' } }, () => t('menu.dataUsers')), key: 'data-users', icon: icon(PeopleOutline) },
    ],
  },
  { label: () => h(RouterLink, { to: { name: 'users' } }, () => t('menu.userManagement')), key: 'users', icon: icon(PersonCircleOutline) },
  {
    label: t('menu.system'),
    key: 'system',
    icon: icon(SettingsOutline),
    children: [
      { label: () => h(RouterLink, { to: { name: 'dict' } }, () => t('menu.dict')), key: 'dict', icon: icon(BookOutline) },
      { label: () => h(RouterLink, { to: { name: 'audit' } }, () => t('menu.auditLog')), key: 'audit', icon: icon(ListOutline) },
    ],
  },
])

const active = computed(() => router.currentRoute.value.name as string)

function onUpdate() {
  // NMenu fires update:value when a leaf is selected (its label renders a
  // RouterLink so navigation already happens). We just need the parent
  // signal to close the drawer.
  emit('navigate')
}
</script>

<template>
  <div class="nav-shell">
    <div class="brand">{{ t('layout.brand') }}</div>
    <NMenu
      :options="menuOptions"
      :value="active"
      :indent="18"
      @update:value="onUpdate"
    />
  </div>
</template>

<style scoped>
.nav-shell {
  display: flex;
  flex-direction: column;
}
.brand {
  padding: 18px 20px;
  font-weight: 600;
  font-size: 16px;
}
</style>
