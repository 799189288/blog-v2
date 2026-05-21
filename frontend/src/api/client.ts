import axios, { type AxiosInstance } from 'axios'
import router from '../router'

const baseURL = import.meta.env.VITE_API_BASE_URL || '/api'

export const api: AxiosInstance = axios.create({
  baseURL,
  timeout: 15000,
})

api.interceptors.response.use(
  res => res,
  err => {
    if (err?.response?.status === 401) {
      router.push({ name: 'home' })
    }
    return Promise.reject(err)
  },
)
