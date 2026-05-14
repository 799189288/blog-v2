import axios, { type AxiosInstance } from 'axios'

const TOKEN_KEY = 'blog_console_token'

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}

export function setToken(token: string | null) {
  if (token === null) localStorage.removeItem(TOKEN_KEY)
  else localStorage.setItem(TOKEN_KEY, token)
}

const baseURL = import.meta.env.VITE_API_BASE_URL || '/api'


export const api: AxiosInstance = axios.create({
  baseURL: baseURL,
  timeout: 15000,
})

api.interceptors.request.use(config => {
  const t = getToken()
  if (t) config.headers.set('Authorization', `Bearer ${t}`)
  return config
})

api.interceptors.response.use(
  r => r,
  err => {
    if (err?.response?.status === 401) {
      setToken(null)
      if (window.location.pathname !== '/login') {
        window.location.href = '/login'
      }
    }
    return Promise.reject(err)
  },
)
