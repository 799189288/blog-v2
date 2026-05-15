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
  // Honeypot — must stay empty. Rendered inside a CSS-hidden div so
  // a human never sees or fills it; spam bots that fill every input
  // get silently dropped by the server.
  website: '',
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
      website: form.website,
    })
    message.success(t('commentForm.submitSuccess'))
    form.author_name = ''
    form.author_email = ''
    form.content = ''
    form.website = ''
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
    <!-- Honeypot. Hidden from real users via aria/CSS but visible to
         naive spam bots that fill every input on the page. -->
    <div class="hp-field" aria-hidden="true">
      <label>
        Website
        <input
          v-model="form.website"
          type="text"
          name="website"
          tabindex="-1"
          autocomplete="off"
        />
      </label>
    </div>
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
  flex-wrap: wrap;
}
@media (max-width: 480px) {
  .actions :deep(.n-button) { flex: 1 1 auto; }
}
/* Hidden honeypot. `display: none` is enough — many bots ignore CSS but
   the ones that do parse it skip non-visible inputs; the ones that don't
   read CSS will fill the field and trip the server-side trap. Either way
   we win. We also use absolute positioning off-screen as a backup since
   some headless browsers actually run CSS. */
.hp-field {
  position: absolute;
  left: -9999px;
  top: -9999px;
  width: 1px;
  height: 1px;
  overflow: hidden;
}
</style>
