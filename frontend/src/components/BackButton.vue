<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NButton } from 'naive-ui'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{ label?: string; fallback?: string }>(),
  { label: '', fallback: '/' },
)

const router = useRouter()
const { t } = useI18n()

const displayLabel = computed(() => props.label || t('common.back'))

function go() {
  if (window.history.state?.back) {
    router.back()
  } else {
    router.push(props.fallback)
  }
}
</script>

<template>
  <NButton text @click="go" class="back-btn">
    <span class="arrow">←</span>
    {{ displayLabel }}
  </NButton>
</template>

<style scoped>
.back-btn {
  margin-bottom: 16px;
}
.arrow {
  margin-right: 6px;
  font-size: 1.1em;
}
</style>
