<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, NLoadingBarProvider, darkTheme, zhCN, enUS, dateZhCN, dateEnUS } from 'naive-ui'
import type { GlobalThemeOverrides } from 'naive-ui'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTheme } from './composables/useTheme'

const { isDark } = useTheme()
const theme = computed(() => (isDark.value ? darkTheme : null))

// Brand color override — brick red (朱红). primaryColorSuppl is what
// Naive UI uses in dark mode in place of primaryColor, so we set both
// to keep brand identity consistent across themes.
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
