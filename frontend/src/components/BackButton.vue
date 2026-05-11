<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NButton } from 'naive-ui'

const props = withDefaults(
  defineProps<{ label?: string; fallback?: string }>(),
  { label: 'Back', fallback: '/' },
)

const router = useRouter()

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
    {{ label }}
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
