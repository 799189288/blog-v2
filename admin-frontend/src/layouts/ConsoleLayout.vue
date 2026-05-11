<script setup lang="ts">
import { computed, h } from 'vue'
import { RouterLink, RouterView, useRouter } from 'vue-router'
import {
  NLayout, NLayoutHeader, NLayoutSider, NLayoutContent,
  NMenu, NSpace, NButton, NIcon,
} from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import {
  StatsChartOutline, DocumentTextOutline, ChatbubbleOutline,
  PricetagsOutline, PeopleOutline, PersonCircleOutline, ListOutline,
  CreateOutline,
} from '@vicons/ionicons5'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const router = useRouter()

function icon(comp: any) { return () => h(NIcon, null, () => h(comp)) }

const menuOptions: MenuOption[] = [
  { label: () => h(RouterLink, { to: { name: 'dashboard' } }, () => 'Dashboard'), key: 'dashboard', icon: icon(StatsChartOutline) },
  {
    label: 'Content',
    key: 'manage',
    icon: icon(CreateOutline),
    children: [
      { label: () => h(RouterLink, { to: { name: 'manage-posts' } }, () => 'Posts'), key: 'manage-posts', icon: icon(DocumentTextOutline) },
      { label: () => h(RouterLink, { to: { name: 'manage-post-new' } }, () => 'New post'), key: 'manage-post-new', icon: icon(CreateOutline) },
      { label: () => h(RouterLink, { to: { name: 'manage-comments' } }, () => 'Comment moderation'), key: 'manage-comments', icon: icon(ChatbubbleOutline) },
    ],
  },
  {
    label: 'Browse data',
    key: 'data',
    icon: icon(ListOutline),
    children: [
      { label: () => h(RouterLink, { to: { name: 'data-posts' } }, () => 'Posts'), key: 'data-posts', icon: icon(DocumentTextOutline) },
      { label: () => h(RouterLink, { to: { name: 'data-comments' } }, () => 'Comments'), key: 'data-comments', icon: icon(ChatbubbleOutline) },
      { label: () => h(RouterLink, { to: { name: 'data-tags' } }, () => 'Tags'), key: 'data-tags', icon: icon(PricetagsOutline) },
      { label: () => h(RouterLink, { to: { name: 'data-users' } }, () => 'Users'), key: 'data-users', icon: icon(PeopleOutline) },
    ],
  },
  { label: () => h(RouterLink, { to: { name: 'users' } }, () => 'User management'), key: 'users', icon: icon(PersonCircleOutline) },
  { label: () => h(RouterLink, { to: { name: 'audit' } }, () => 'Audit log'), key: 'audit', icon: icon(ListOutline) },
]

const active = computed(() => router.currentRoute.value.name as string)

function onLogout() {
  auth.logout()
  router.push({ name: 'login' })
}
</script>

<template>
  <NLayout has-sider style="min-height: 100vh">
    <NLayoutSider bordered :width="240" :collapsed-width="64" show-trigger="bar">
      <div class="brand">Blog Console</div>
      <NMenu :options="menuOptions" :value="active" :indent="18" />
    </NLayoutSider>
    <NLayout>
      <NLayoutHeader bordered style="padding: 12px 24px;">
        <NSpace justify="end" align="center">
          <span v-if="auth.user">{{ auth.user.username }} ({{ auth.user.role }})</span>
          <NButton size="small" @click="onLogout">Log out</NButton>
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
</style>
