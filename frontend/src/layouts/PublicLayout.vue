<script setup lang="ts">
import { RouterLink, RouterView, useRouter } from "vue-router";
import {
  NLayout,
  NLayoutHeader,
  NLayoutContent,
  NLayoutFooter,
  NInput,
  NDropdown,
  NButton,
} from "naive-ui";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { setLocale, type Locale } from "../i18n";
import { useTheme, type ThemeMode } from "../composables/useTheme";
import { useBreakpoint } from "../composables/useBreakpoint";

const router = useRouter();
const query = ref("");
const { t, locale } = useI18n();
const { mode: themeMode, cycle: cycleTheme } = useTheme();
const { isNarrow } = useBreakpoint();

function onSearch() {
  const q = query.value.trim();
  if (q) router.push({ name: "search", query: { q } });
}

const langOptions = computed(() => [
  { key: "en", label: t("language.en") },
  { key: "zh", label: t("language.zh") },
]);

const currentLangLabel = computed(() =>
  locale.value === "zh" ? t("language.zh") : t("language.en"),
);

function onLangSelect(key: string) {
  setLocale(key as Locale);
}

// Short label per state. The `title` attr still gets the long
// description plus the "click to switch" hint.
const themeLabel = computed(() => {
  const m: ThemeMode = themeMode.value;
  return t(`theme.${m}Short`);
});
const themeTitle = computed(() => {
  const m: ThemeMode = themeMode.value;
  return t(`theme.${m}`) + " — " + t("theme.clickToCycle");
});
</script>

<template>
  <NLayout class="root-layout">
    <NLayoutHeader bordered style="padding: 14px 24px">
      <div class="header-inner">
        <div class="header-left">
          <RouterLink :to="{ name: 'home' }" class="brand">{{
            t("layout.brand")
          }}</RouterLink>
          <nav class="nav">
            <RouterLink
              :to="{ name: 'home' }"
              class="nav-link"
              :exact-active-class="'is-active'"
              active-class=""
              >{{ t("layout.home") }}</RouterLink
            >
            <RouterLink
              :to="{ name: 'archive' }"
              class="nav-link"
              active-class="is-active"
              >{{ t("layout.archive") }}</RouterLink
            >
          </nav>
        </div>
        <div class="header-right">
          <NInput
            v-if="!isNarrow"
            v-model:value="query"
            :placeholder="t('layout.searchPlaceholder')"
            clearable
            size="small"
            class="header-search"
            @keyup.enter="onSearch"
          />
          <RouterLink
            v-else
            :to="{ name: 'search' }"
            class="search-link"
            :title="t('layout.searchPlaceholder')"
            :aria-label="t('layout.searchPlaceholder')"
            >{{ t("common.search") }}</RouterLink
          >
          <NButton
            class="theme-btn"
            :title="themeTitle"
            :aria-label="themeTitle"
            @click="cycleTheme"
          >
            {{ themeLabel }}
          </NButton>
          <NDropdown
            trigger="click"
            :options="langOptions"
            @select="onLangSelect"
          >
            <NButton class="lang-btn">{{ currentLangLabel }} ▾</NButton>
          </NDropdown>
        </div>
      </div>
    </NLayoutHeader>

    <NLayoutContent class="main-content" style="padding: 32px 24px">
      <div class="content-wrap">
        <RouterView />
      </div>
    </NLayoutContent>

    <NLayoutFooter
      bordered
      style="padding: 16px 24px; text-align: center; font-size: 14px"
    >
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
  flex-wrap: wrap;
  row-gap: 8px;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.header-search {
  width: 220px;
  max-width: 50vw;
}
.search-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px 10px;
  border: 1px solid rgba(127, 127, 127, 0.3);
  border-radius: 4px;
  text-decoration: none;
  color: inherit;
  font-size: 13px;
}
.search-link:hover {
  border-color: rgba(127, 127, 127, 0.6);
}
.brand {
  font-family: "EB Garamond", "Noto Serif SC", Georgia, serif;
  font-weight: 600;
  font-size: 20px;
  text-decoration: none;
  letter-spacing: -0.005em;
}
.header-left {
  display: flex;
  align-items: baseline;
  gap: 24px;
}
.nav {
  display: flex;
  gap: 16px;
}
.nav-link {
  text-decoration: none;
  font-size: 14px;
  opacity: 0.65;
  color: inherit;
  transition:
    opacity 0.15s,
    color 0.15s;
}
.nav-link:hover {
  opacity: 1;
}
.nav-link.is-active {
  opacity: 1;
  color: var(--brand-color, #c0392b);
  font-weight: 500;
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
.theme-btn:hover {
  border-color: rgba(127, 127, 127, 0.6);
}
.content-wrap {
  max-width: 960px;
  margin: 0 auto;
}

@media (max-width: 768px) {
  .header-inner {
    gap: 10px;
  }
  .header-left {
    gap: 12px;
  }
  .brand {
    font-size: 16px;
  }
  .main-content {
    padding: 20px 16px !important;
  }
  :deep(.n-layout-header) {
    padding: 10px 14px !important;
  }
  :deep(.n-layout-footer) {
    padding: 12px 14px !important;
  }
}
@media (max-width: 480px) {
  .header-right {
    gap: 8px;
  }
  .nav-link {
    font-size: 13px;
  }
  .main-content {
    padding: 16px 12px !important;
  }
}
</style>
