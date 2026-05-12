<script setup lang="ts">
import { reactive, ref } from 'vue'
import { NForm, NFormItem, NInput, NButton, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { submit } from '../api/comments'

const props = withDefaults(
  defineProps<{
    slug: string
    parentId?: number | null
    replyToName?: string | null
    compact?: boolean
  }>(),
  { parentId: null, replyToName: null, compact: false },
)
const emit = defineEmits<{ submitted: []; cancel: [] }>()

const message = useMessage()
const { t } = useI18n()
const submitting = ref(false)
const form = reactive({
  author_name: '',
  author_email: '',
  content: '',
})

async function onSubmit() {
  if (!form.author_name.trim() || !form.content.trim()) {
    message.warning(t('commentForm.requiredError'))
    return
  }
  submitting.value = true
  try {
    await submit(props.slug, {
      author_name: form.author_name.trim(),
      author_email: form.author_email.trim() || undefined,
      content: form.content.trim(),
      parent_id: props.parentId ?? undefined,
    })
    message.success(t('commentForm.submitSuccess'))
    form.author_name = ''
    form.author_email = ''
    form.content = ''
    emit('submitted')
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('commentForm.submitFail'))
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <NForm label-placement="top" @submit.prevent="onSubmit">
    <div v-if="replyToName" class="reply-hint">
      {{ t('commentForm.replyingTo') }}{{ replyToName }}
    </div>
    <NFormItem :label="t('commentForm.name')" required>
      <NInput v-model:value="form.author_name" :placeholder="t('commentForm.namePlaceholder')" maxlength="64" />
    </NFormItem>
    <NFormItem :label="t('commentForm.email')">
      <NInput v-model:value="form.author_email" :placeholder="t('commentForm.emailPlaceholder')" />
    </NFormItem>
    <NFormItem :label="t('commentForm.content')" required>
      <NInput v-model:value="form.content" type="textarea" :rows="compact ? 3 : 4" :placeholder="t('commentForm.contentPlaceholder')" />
    </NFormItem>
    <div class="actions">
      <NButton type="primary" :loading="submitting" attr-type="submit">
        {{ t('common.submit') }}
      </NButton>
      <NButton v-if="parentId" quaternary @click="emit('cancel')">
        {{ t('common.cancel') }}
      </NButton>
    </div>
  </NForm>
</template>

<style scoped>
.reply-hint {
  font-size: 13px;
  opacity: 0.7;
  margin-bottom: 8px;
}
.actions {
  display: flex;
  gap: 8px;
}
</style>
