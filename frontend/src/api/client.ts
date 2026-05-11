import axios, { type AxiosInstance } from 'axios'

export const api: AxiosInstance = axios.create({
  baseURL: '/api',
  timeout: 15000,
})
