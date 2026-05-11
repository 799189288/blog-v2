<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NCard, NForm, NFormItem, NInput, NButton, useMessage } from 'naive-ui'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()
const message = useMessage()

const username = ref('')
const password = ref('')
const loading = ref(false)

async function onSubmit() {
  if (!username.value || !password.value) {
    message.warning('Username and password required')
    return
  }
  loading.value = true
  try {
    await auth.login(username.value, password.value)
    const next = (route.query.next as string) || '/dashboard'
    router.replace(next)
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? 'Login failed')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="wrap">
    <NCard title="Blog Console" style="width: 380px;">
      <p style="margin-top: 0; opacity: 0.7; font-size: 14px;">Sign in with an admin account.</p>
      <NForm @submit.prevent="onSubmit">
        <NFormItem label="Username">
          <NInput v-model:value="username" autocomplete="username" />
        </NFormItem>
        <NFormItem label="Password">
          <NInput v-model:value="password" type="password" show-password-on="click" />
        </NFormItem>
        <NButton type="primary" block :loading="loading" attr-type="submit">
          Sign in
        </NButton>
      </NForm>
    </NCard>
  </div>
</template>

<style scoped>
.wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 24px;
}
</style>
