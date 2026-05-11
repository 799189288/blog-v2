import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getToken, setToken } from '../api/client'
import * as authApi from '../api/auth'
import type { AuthUser } from '../types'

const USER_KEY = 'blog_console_user'

function loadUser(): AuthUser | null {
  try {
    const raw = localStorage.getItem(USER_KEY)
    return raw ? (JSON.parse(raw) as AuthUser) : null
  } catch {
    return null
  }
}

function saveUser(u: AuthUser | null) {
  if (u === null) localStorage.removeItem(USER_KEY)
  else localStorage.setItem(USER_KEY, JSON.stringify(u))
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(getToken())
  const user = ref<AuthUser | null>(loadUser())

  async function login(username: string, password: string) {
    const res = await authApi.login(username, password)
    setToken(res.token)
    saveUser(res.user)
    token.value = res.token
    user.value = res.user
  }

  function logout() {
    setToken(null)
    saveUser(null)
    token.value = null
    user.value = null
  }

  return { token, user, login, logout }
})
