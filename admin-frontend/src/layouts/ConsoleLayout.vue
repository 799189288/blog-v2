<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterView, useRouter, useRoute } from 'vue-router'
import {
  NLayout, NLayoutHeader, NLayoutSider, NLayoutContent,
  NSpace, NButton, NDropdown, NDrawer, NDrawerContent,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'
import { setLocale, type Locale } from '../i18n'
import { useTheme, type ThemeMode } from '../composables/useTheme'
import { useBreakpoint } from '../composables/useBreakpoint'
import AdminNavMenu from '../components/AdminNavMenu.vue'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()
const { t, locale } = useI18n()
const { mode: themeMode, cycle: cycleTheme } = useTheme()
const { isMobile } = useBreakpoint()

const drawerOpen = ref(false)

// Close the drawer whenever we navigate (a menu selection triggered the
// route change) or when the viewport grows past the mobile breakpoint
// (e.g. user rotates a tablet or resizes the desktop window).
watch(() => route.fullPath, () => { drawerOpen.value = false })
watch(isMobile, m => { if (!m) drawerOpen.value = false })

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

const themeLabel = computed(() => {
  const m: ThemeMode = themeMode.value
  return t(`theme.${m}Short`)
})
const themeTitle = computed(() => {
  const m: ThemeMode = themeMode.value
  return t(`theme.${m}`) + ' — ' + t('theme.clickToCycle')
})
</script>

<template>
  <NLayout has-sider style="min-height: 100vh">
    <NLayoutSider
      v-if="!isMobile"
      bordered
      :width="240"
      :collapsed-width="64"
      show-trigger="bar"
    >
      <AdminNavMenu />
    </NLayoutSider>
    <NLayout>
      <NLayoutHeader bordered class="admin-header">
        <div class="header-left">
          <NButton
            v-if="isMobile"
            quaternary
            size="small"
            class="hamburger"
            :aria-label="t('mobile.menu')"
            @click="drawerOpen = true"
          >{{ t('mobile.menu') }}</NButton>
        </div>
        <NSpace class="header-right" align="center" :wrap="true">
          <button
            type="button"
            class="theme-btn"
            :title="themeTitle"
            :aria-label="themeTitle"
            @click="cycleTheme"
          >{{ themeLabel }}</button>
          <NDropdown trigger="click" :options="langOptions" @select="onLangSelect">
            <button type="button" class="lang-btn">{{ currentLangLabel }} ▾</button>
          </NDropdown>
          <span v-if="auth.user && !isMobile" class="user-tag">
            {{ auth.user.username }} ({{ auth.user.role }})
          </span>
          <NButton size="small" @click="onLogout">{{ t('layout.logout') }}</NButton>
        </NSpace>
      </NLayoutHeader>
      <NLayoutContent class="admin-content">
        <RouterView />
      </NLayoutContent>
    </NLayout>
  </NLayout>

  <NDrawer
    v-if="isMobile"
    v-model:show="drawerOpen"
    placement="left"
    :width="280"
  >
    <NDrawerContent :title="t('layout.brand')" closable>
      <AdminNavMenu @navigate="drawerOpen = false" />
    </NDrawerContent>
  </NDrawer>
</template>

<style scoped>
.admin-header {
  padding: 12px 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.header-left { display: flex; align-items: center; }
.header-right { flex: 0 1 auto; justify-content: flex-end; }
.hamburger {
  line-height: 1;
  padding: 0 12px;
  height: 32px;
}
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
.user-tag { font-size: 13px; opacity: 0.75; }
.admin-content { padding: 24px; }
@media (max-width: 768px) {
  .admin-header { padding: 10px 14px; }
  .admin-content { padding: 16px 12px; }
}
</style>
