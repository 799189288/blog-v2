<script setup lang="ts">
import { reactive, ref } from 'vue'
import { NForm, NFormItem, NInput, NButton, useMessage } from 'naive-ui'
import { submit } from '../api/comments'

const props = defineProps<{ slug: string }>()
const emit = defineEmits<{ submitted: [] }>()

const message = useMessage()
const submitting = ref(false)
const form = reactive({
  author_name: '',
  author_email: '',
  content: '',
})

async function onSubmit() {
  if (!form.author_name.trim() || !form.content.trim()) {
    message.warning('Name and comment are required')
    return
  }
  submitting.value = true
  try {
    await submit(props.slug, {
      author_name: form.author_name.trim(),
      author_email: form.author_email.trim() || undefined,
      content: form.content.trim(),
    })
    message.success('Comment submitted — pending moderation.')
    form.author_name = ''
    form.author_email = ''
    form.content = ''
    emit('submitted')
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? 'Failed to submit')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <NForm label-placement="top" @submit.prevent="onSubmit">
    <NFormItem label="Name" required>
      <NInput v-model:value="form.author_name" placeholder="Your name" maxlength="64" />
    </NFormItem>
    <NFormItem label="Email (optional, not shown)">
      <NInput v-model:value="form.author_email" placeholder="you@example.com" />
    </NFormItem>
    <NFormItem label="Comment" required>
      <NInput v-model:value="form.content" type="textarea" :rows="4" placeholder="Say something..." />
    </NFormItem>
    <NButton type="primary" :loading="submitting" attr-type="submit">
      Submit
    </NButton>
  </NForm>
</template>
