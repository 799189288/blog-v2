<script setup lang="ts">
import { RouterLink, RouterView, useRouter } from 'vue-router'
import { NLayout, NLayoutHeader, NLayoutContent, NLayoutFooter, NInput, NDropdown } from 'naive-ui'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLocale, type Locale } from '../i18n'

const router = useRouter()
const query = ref('')
const { t, locale } = useI18n()

function onSearch() {
  const q = query.value.trim()
  if (q) router.push({ name: 'search', query: { q } })
}

const langOptions = computed(() => [
  { key: 'en', label: t('language.en') },
  { key: 'zh', label: t('language.zh') },
])

const currentLangLabel = computed(() => (locale.value === 'zh' ? t('language.zh') : t('language.en')))

function onLangSelect(key: string) {
  setLocale(key as Locale)
}
</script>

<template>
  <NLayout class="root-layout">
    <NLayoutHeader bordered style="padding: 14px 24px;">
      <div class="header-inner">
        <RouterLink :to="{ name: 'home' }" class="brand">{{ t('layout.brand') }}</RouterLink>
        <div class="header-right">
          <NInput
            v-model:value="query"
            :placeholder="t('layout.searchPlaceholder')"
            clearable
            size="small"
            style="width: 220px"
            @keyup.enter="onSearch"
          />
          <NDropdown trigger="click" :options="langOptions" @select="onLangSelect">
            <button type="button" class="lang-btn">{{ currentLangLabel }} ▾</button>
          </NDropdown>
        </div>
      </div>
    </NLayoutHeader>

    <NLayoutContent class="main-content" style="padding: 32px 24px;">
      <div class="content-wrap">
        <RouterView />
      </div>
    </NLayoutContent>

    <NLayoutFooter bordered style="padding: 16px 24px; text-align: center; font-size: 14px;">
      &copy; {{ new Date().getFullYear() }} Leo
    </NLayoutFooter>
  </NLayout>
</template>

<style scoped>
.root-layout {
  min-height: 100vh;
}
/* Naive UI wraps NLayout's children in `.n-layout-scroll-container`.
   Make that the flex column so the footer sits at the bottom. */
.root-layout > :deep(.n-layout-scroll-container) {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
.main-content {
  flex: 1 0 auto;
}
.header-inner {
  max-width: 960px;
  margin: 0 auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}
.brand {
  font-weight: 600;
  font-size: 18px;
  text-decoration: none;
}
.lang-btn {
  background: transparent;
  border: 1px solid rgba(127, 127, 127, 0.3);
  border-radius: 4px;
  padding: 4px 10px;
  cursor: pointer;
  font: inherit;
  color: inherit;
}
.lang-btn:hover {
  border-color: rgba(127, 127, 127, 0.6);
}
.content-wrap {
  max-width: 960px;
  margin: 0 auto;
}
</style>
