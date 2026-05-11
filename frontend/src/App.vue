<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, NLoadingBarProvider, darkTheme } from 'naive-ui'
import { computed, ref, onMounted } from 'vue'

const dark = ref(false)
onMounted(() => {
  const mq = window.matchMedia('(prefers-color-scheme: dark)')
  dark.value = mq.matches
  mq.addEventListener('change', e => { dark.value = e.matches })
})
const theme = computed(() => (dark.value ? darkTheme : null))
</script>

<template>
  <NConfigProvider :theme="theme">
    <NLoadingBarProvider>
      <NDialogProvider>
        <NMessageProvider>
          <RouterView />
        </NMessageProvider>
      </NDialogProvider>
    </NLoadingBarProvider>
  </NConfigProvider>
</template>
