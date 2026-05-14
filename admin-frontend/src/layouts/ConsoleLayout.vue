<script setup lang="ts">
import { computed, h } from 'vue'
import { RouterLink, RouterView, useRouter } from 'vue-router'
import {
  NLayout, NLayoutHeader, NLayoutSider, NLayoutContent,
  NMenu, NSpace, NButton, NIcon, NDropdown,
} from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import {
  StatsChartOutline, DocumentTextOutline, ChatbubbleOutline,
  PricetagsOutline, PeopleOutline, PersonCircleOutline, ListOutline,
  CreateOutline, BookOutline, SettingsOutline,
} from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'
import { setLocale, type Locale } from '../i18n'
import { useTheme, type ThemeMode } from '../composables/useTheme'

const auth = useAuthStore()
const router = useRouter()
const { t, locale } = useI18n()
const { mode: themeMode, cycle: cycleTheme } = useTheme()

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

function onLogout() {
  auth.logout()
  router.push({ name: 'login' })
}

const langOptions = computed(() => [
  { key: 'en', label: t('language.en') },
  { key: 'zh', label: t('language.zh') },
])
const currentLangLabel = computed(() => (locale.value === 'zh' ? t('language.zh') : t('language.en')))
function onLangSelect(key: string) {
  setLocale(key as Locale)
}

const themeIcon = computed(() => {
  const m: ThemeMode = themeMode.value
  return m === 'auto' ? '🖥' : m === 'light' ? '☀' : '☾'
})
const themeTitle = computed(() => {
  const m: ThemeMode = themeMode.value
  return t(`theme.${m}`) + ' — ' + t('theme.clickToCycle')
})
</script>

<template>
  <NLayout has-sider style="min-height: 100vh">
    <NLayoutSider bordered :width="240" :collapsed-width="64" show-trigger="bar">
      <div class="brand">{{ t('layout.brand') }}</div>
      <NMenu :options="menuOptions" :value="active" :indent="18" />
    </NLayoutSider>
    <NLayout>
      <NLayoutHeader bordered style="padding: 12px 24px;">
        <NSpace justify="end" align="center">
          <button
            type="button"
            class="theme-btn"
            :title="themeTitle"
            :aria-label="themeTitle"
            @click="cycleTheme"
          >{{ themeIcon }}</button>
          <NDropdown trigger="click" :options="langOptions" @select="onLangSelect">
            <button type="button" class="lang-btn">{{ currentLangLabel }} ▾</button>
          </NDropdown>
          <span v-if="auth.user">{{ auth.user.username }} ({{ auth.user.role }})</span>
          <NButton size="small" @click="onLogout">{{ t('layout.logout') }}</NButton>
        </NSpace>
      </NLayoutHeader>
      <NLayoutContent style="padding: 24px;">
        <RouterView />
      </NLayoutContent>
    </NLayout>
  </NLayout>
</template>

<style scoped>
.brand { padding: 18px 20px; font-weight: 600; font-size: 16px; }
.lang-btn,
.theme-btn {
  background: transparent;
  border: 1px solid rgba(127, 127, 127, 0.3);
  border-radius: 4px;
  padding: 4px 10px;
  cursor: pointer;
  font: inherit;
  color: inherit;
}
.lang-btn:hover,
.theme-btn:hover { border-color: rgba(127, 127, 127, 0.6); }
.theme-btn {
  min-width: 34px;
  text-align: center;
  font-size: 16px;
  line-height: 1;
  padding: 4px 8px;
}
</style>
