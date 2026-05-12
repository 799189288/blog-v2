import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const routes = [
  { path: '/login', name: 'login', component: () => import('../views/LoginView.vue') },
  {
    path: '/',
    component: () => import('../layouts/ConsoleLayout.vue'),
    meta: { requiresAuth: true },
    children: [
      { path: '', redirect: { name: 'dashboard' } },
      { path: 'dashboard', name: 'dashboard', component: () => import('../views/DashboardView.vue') },
      // Content management (write)
      { path: 'manage/posts', name: 'manage-posts', component: () => import('../views/manage/PostListView.vue') },
      { path: 'manage/posts/new', name: 'manage-post-new', component: () => import('../views/manage/PostEditView.vue') },
      { path: 'manage/posts/:id/edit', name: 'manage-post-edit', component: () => import('../views/manage/PostEditView.vue'), props: true },
      { path: 'manage/comments', name: 'manage-comments', component: () => import('../views/manage/CommentModerationView.vue') },
      { path: 'manage/tags', name: 'manage-tags', component: () => import('../views/manage/TagListView.vue') },
      // Raw data browse (read-only)
      { path: 'data/posts', name: 'data-posts', component: () => import('../views/data/PostsTableView.vue') },
      { path: 'data/comments', name: 'data-comments', component: () => import('../views/data/CommentsTableView.vue') },
      { path: 'data/tags', name: 'data-tags', component: () => import('../views/data/TagsTableView.vue') },
      { path: 'data/users', name: 'data-users', component: () => import('../views/data/UsersTableView.vue') },
      { path: 'users', name: 'users', component: () => import('../views/UsersView.vue') },
      { path: 'dict', name: 'dict', component: () => import('../views/DictManageView.vue') },
      { path: 'audit', name: 'audit', component: () => import('../views/AuditView.vue') },
    ],
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior() { return { top: 0 } },
})

router.beforeEach(to => {
  if (to.meta.requiresAuth) {
    const auth = useAuthStore()
    if (!auth.token) {
      return { name: 'login', query: { next: to.fullPath } }
    }
  }
  return true
})

export default router
