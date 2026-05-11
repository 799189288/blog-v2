<script setup lang="ts">
import { RouterLink, RouterView, useRouter } from 'vue-router'
import { NLayout, NLayoutHeader, NLayoutContent, NLayoutFooter, NInput } from 'naive-ui'
import { ref } from 'vue'

const router = useRouter()
const query = ref('')

function onSearch() {
  const q = query.value.trim()
  if (q) router.push({ name: 'search', query: { q } })
}
</script>

<template>
  <NLayout class="root-layout">
    <NLayoutHeader bordered style="padding: 14px 24px;">
      <div class="header-inner">
        <RouterLink :to="{ name: 'home' }" class="brand">Home</RouterLink>
        <NInput
          v-model:value="query"
          placeholder="Search posts..."
          clearable
          size="small"
          style="width: 220px"
          @keyup.enter="onSearch"
        />
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
.brand {
  font-weight: 600;
  font-size: 18px;
  text-decoration: none;
}
.content-wrap {
  max-width: 960px;
  margin: 0 auto;
}
</style>
