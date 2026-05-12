<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, NLoadingBarProvider, darkTheme, zhCN, enUS, dateZhCN, dateEnUS } from 'naive-ui'
import { computed, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

const dark = ref(false)
onMounted(() => {
  const mq = window.matchMedia('(prefers-color-scheme: dark)')
  dark.value = mq.matches
  mq.addEventListener('change', e => { dark.value = e.matches })
})
const theme = computed(() => (dark.value ? darkTheme : null))

const { locale } = useI18n()
const naiveLocale = computed(() => (locale.value === 'zh' ? zhCN : enUS))
const naiveDateLocale = computed(() => (locale.value === 'zh' ? dateZhCN : dateEnUS))
</script>

<template>
  <NConfigProvider :theme="theme" :locale="naiveLocale" :date-locale="naiveDateLocale">
    <NLoadingBarProvider>
      <NDialogProvider>
        <NMessageProvider>
          <RouterView />
        </NMessageProvider>
      </NDialogProvider>
    </NLoadingBarProvider>
  </NConfigProvider>
</template>
