<script setup lang="ts">
import { computed } from 'vue'
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NLoadingBarProvider,
  darkTheme,
  zhCN, enUS, dateZhCN, dateEnUS,
} from 'naive-ui'
import type { GlobalThemeOverrides } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useTheme } from './composables/useTheme'

const { isDark } = useTheme()
const theme = computed(() => (isDark.value ? darkTheme : null))

// Brand color override — matches the public site (frontend/) so the
// admin and the blog share one visual identity. primaryColorSuppl is
// used in place of primaryColor under darkTheme.
const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#c0392b',
    primaryColorHover: '#d3543f',
    primaryColorPressed: '#a02d22',
    primaryColorSuppl: '#c0392b',
  },
}

const { locale } = useI18n()
const naiveLocale = computed(() => (locale.value === 'zh' ? zhCN : enUS))
const naiveDateLocale = computed(() => (locale.value === 'zh' ? dateZhCN : dateEnUS))
</script>

<template>
  <NConfigProvider :theme="theme" :theme-overrides="themeOverrides" :locale="naiveLocale" :date-locale="naiveDateLocale">
    <NLoadingBarProvider>
      <NDialogProvider>
        <NMessageProvider>
          <RouterView />
        </NMessageProvider>
      </NDialogProvider>
    </NLoadingBarProvider>
  </NConfigProvider>
</template>
