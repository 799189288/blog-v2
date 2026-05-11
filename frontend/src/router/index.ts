import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    component: () => import('../layouts/PublicLayout.vue'),
    children: [
      { path: '', name: 'home', component: () => import('../views/HomeView.vue') },
      { path: 'post/:slug', name: 'post', component: () => import('../views/PostView.vue'), props: true },
      { path: 'tag/:slug', name: 'tag', component: () => import('../views/TagView.vue'), props: true },
      { path: 'search', name: 'search', component: () => import('../views/SearchView.vue') },
    ],
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior() { return { top: 0 } },
})

export default router
