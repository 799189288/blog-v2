<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
  NForm, NFormItem, NInput, NDynamicTags, NSelect, NButton, NSpace, NSpin, NAlert,
  NCard, NResult, useMessage,
} from 'naive-ui'
import { MdEditor, config } from 'md-editor-v3'
import katex from 'katex'
import 'katex/dist/katex.min.css'
import { useI18n } from 'vue-i18n'
import * as postsApi from '../../api/posts'
import { useDictStore } from '../../stores/dict'
import { useTheme } from '../../composables/useTheme'
import { useBreakpoint } from '../../composables/useBreakpoint'

// Enable $...$ / $$...$$ math rendering via KaTeX. md-editor-v3 looks
// up the katex instance from this global config the first time it
// renders a math block; safe to call on every component mount.
config({
  editorExtensions: {
    katex: { instance: katex },
  },
})

const props = defineProps<{ id?: string }>()
const router = useRouter()
const message = useMessage()
const { t, locale } = useI18n()
const dict = useDictStore()
const { isDark } = useTheme()
const { isMobile } = useBreakpoint()

const editorTheme = computed<'light' | 'dark'>(() => (isDark.value ? 'dark' : 'light'))
const editorLang = computed<'zh-CN' | 'en-US'>(() => (locale.value === 'zh' ? 'zh-CN' : 'en-US'))

const isEdit = computed(() => !!props.id)
const loading = ref(false)
const saving = ref(false)

// Public site origin. Defaults to the dev port; override with
// VITE_PUBLIC_SITE_URL=https://yourdomain.com at build time for prod.
const PUBLIC_SITE_URL = (import.meta.env.VITE_PUBLIC_SITE_URL as string | undefined) ?? 'http://localhost:5178'

const form = reactive({
  title: '',
  slug: '',
  excerpt: '',
  content_md: '',
  status: 'draft' as 'draft' | 'published',
  tags: [] as string[],
})

// Only meaningful on drafts; cleared on every fresh load.
const previewToken = ref<string | null>(null)

const previewUrl = computed(() => {
  if (form.status !== 'draft' || !previewToken.value || !form.slug) return null
  const slug = encodeURIComponent(form.slug)
  return `${PUBLIC_SITE_URL}/post/${slug}?token=${previewToken.value}`
})

const statusOptions = computed(() => dict.options('post.status').value)

/// MdEditor's onUploadImg contract: receives a File[] and a callback the
/// editor invokes with one URL per file. We upload sequentially so a
/// failed one stops the chain instead of leaving inserted-but-broken
/// placeholders; md-editor-v3 inserts ![](url) for every URL returned.
async function onUploadImg(files: File[], callback: (urls: string[]) => void) {
  const urls: string[] = []
  for (const f of files) {
    try {
      const { url } = await postsApi.uploadImage(f)
      urls.push(url)
    } catch (e: any) {
      message.error(e?.response?.data?.error ?? t('postEdit.uploadFailed'))
      break
    }
  }
  callback(urls)
}

async function load() {
  if (!props.id) return
  loading.value = true
  previewToken.value = null
  try {
    const p = await postsApi.adminGet(Number(props.id))
    form.title = p.title
    form.slug = p.slug
    form.excerpt = p.excerpt ?? ''
    form.content_md = p.content_md
    form.status = p.status
    form.tags = p.tags.map(tg => tg.name)
    previewToken.value = p.preview_token
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('postEdit.loadFailed'))
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await dict.ensure('post.status')
  await load()
})

async function save() {
  if (!form.title.trim() || !form.content_md.trim()) {
    message.warning(t('postEdit.requiredError'))
    return
  }
  saving.value = true
  try {
    const payload = {
      title: form.title.trim(),
      slug: form.slug.trim() || undefined,
      excerpt: form.excerpt.trim() || undefined,
      content_md: form.content_md,
      status: form.status,
      tags: form.tags,
    }
    if (isEdit.value) {
      const updated = await postsApi.adminUpdate(Number(props.id), payload)
      previewToken.value = updated.preview_token
      message.success(t('postEdit.saved'))
    } else {
      const created = await postsApi.adminCreate(payload)
      previewToken.value = created.preview_token
      message.success(t('postEdit.created'))
      router.replace({ name: 'manage-post-edit', params: { id: created.id } })
    }
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('common.saveFailed'))
  } finally {
    saving.value = false
  }
}

async function copyPreviewUrl() {
  if (!previewUrl.value) return
  try {
    await navigator.clipboard.writeText(previewUrl.value)
    message.success(t('postEdit.previewCopied'))
  } catch {
    message.error(t('postEdit.previewCopyFailed'))
  }
}
</script>

<template>
  <NSpin :show="loading">
    <div v-if="isMobile" class="mobile-block">
      <NCard>
        <NResult
          status="info"
          :title="t('mobile.editorBlockTitle')"
          :description="t('mobile.editorBlockDesc')"
        >
          <template #footer>
            <NSpace justify="center" :wrap="true">
              <NButton @click="router.push({ name: 'manage-posts' })">
                {{ t('common.backToList') }}
              </NButton>
              <NButton
                v-if="isEdit && form.slug && form.status === 'published'"
                tag="a"
                :href="`${PUBLIC_SITE_URL}/post/${form.slug}`"
                target="_blank"
                rel="noopener"
              >
                {{ t('mobile.viewPost') }}
              </NButton>
            </NSpace>
          </template>
        </NResult>
      </NCard>
    </div>
    <template v-else>
      <h2 class="page-title">{{ isEdit ? t('postEdit.titleEdit') : t('postEdit.titleNew') }}</h2>
      <NForm label-placement="top">
        <NFormItem :label="t('postEdit.fields.title')" required>
          <NInput v-model:value="form.title" maxlength="200" />
        </NFormItem>
        <NFormItem :label="t('postEdit.fields.slug')">
          <NInput v-model:value="form.slug" :placeholder="t('postEdit.fields.slugPlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('postEdit.fields.excerpt')">
          <NInput v-model:value="form.excerpt" type="textarea" :rows="2" />
        </NFormItem>
        <NFormItem :label="t('postEdit.fields.tags')">
          <NDynamicTags v-model:value="form.tags" />
        </NFormItem>
        <NFormItem :label="t('postEdit.fields.status')">
          <NSelect v-model:value="form.status" :options="statusOptions" style="max-width: 200px;" />
        </NFormItem>
        <NAlert
          v-if="previewUrl"
          type="info"
          :show-icon="false"
          style="margin-bottom: 18px;"
        >
          <div class="preview-row">
            <div class="preview-text">
              <div class="preview-label">{{ t('postEdit.previewLabel') }}</div>
              <a :href="previewUrl" target="_blank" rel="noopener" class="preview-link">{{ previewUrl }}</a>
            </div>
            <NButton size="small" @click="copyPreviewUrl">{{ t('postEdit.previewCopy') }}</NButton>
          </div>
        </NAlert>
        <NFormItem :label="t('postEdit.fields.content')" required>
          <MdEditor
            v-model="form.content_md"
            :preview-only="false"
            :on-upload-img="onUploadImg"
            :theme="editorTheme"
            :language="editorLang"
            style="height: 540px; width: 100%;"
          />
        </NFormItem>
        <NSpace>
          <NButton type="primary" :loading="saving" @click="save">{{ t('common.save') }}</NButton>
          <NButton @click="router.push({ name: 'manage-posts' })">{{ t('common.cancel') }}</NButton>
        </NSpace>
      </NForm>
    </template>
  </NSpin>
</template>

<style scoped>
.page-title { margin-top: 0; }
.mobile-block { padding: 8px 0; }
.preview-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.preview-text {
  min-width: 0;
  flex: 1 1 200px;
}
.preview-label {
  font-size: 12px;
  opacity: 0.7;
  margin-bottom: 2px;
}
.preview-link {
  font-family: ui-monospace, "Cascadia Mono", Menlo, monospace;
  font-size: 13px;
  word-break: break-all;
}
</style>
