<script setup lang="ts">
import { onMounted, ref, watch, nextTick } from 'vue'
import hljs from 'highlight.js'

const props = defineProps<{ html: string }>()
const container = ref<HTMLElement | null>(null)

function highlight() {
  if (!container.value) return
  container.value.querySelectorAll<HTMLElement>('pre code').forEach(el => {
    if (!el.dataset.hl) {
      hljs.highlightElement(el)
      el.dataset.hl = '1'
    }
  })
}

onMounted(highlight)
watch(() => props.html, async () => {
  await nextTick()
  highlight()
})
</script>

<template>
  <div ref="container" class="post-content" v-html="html" />
</template>

<style scoped>
.post-content {
  line-height: 1.7;
  font-size: 16px;
}
.post-content :deep(h1),
.post-content :deep(h2),
.post-content :deep(h3),
.post-content :deep(h4) {
  margin: 1.4em 0 0.6em;
  line-height: 1.3;
}
.post-content :deep(p) { margin: 0.8em 0; }
.post-content :deep(pre) {
  margin: 1em 0;
  border-radius: 6px;
  overflow-x: auto;
  font-size: 13.5px;
  border: 1px solid rgba(127, 127, 127, 0.2);
}
.post-content :deep(pre > code.hljs) {
  padding: 14px 16px;
  display: block;
}
.post-content :deep(:not(pre) > code) {
  background: rgba(127, 127, 127, 0.15);
  padding: 2px 5px;
  border-radius: 3px;
  font-size: 0.92em;
}
.post-content :deep(blockquote) {
  border-left: 4px solid rgba(127, 127, 127, 0.3);
  margin: 1em 0;
  padding: 0.4em 1em;
  opacity: 0.85;
}
.post-content :deep(img) { max-width: 100%; height: auto; }
.post-content :deep(table) { border-collapse: collapse; margin: 1em 0; }
.post-content :deep(th),
.post-content :deep(td) {
  border: 1px solid rgba(127, 127, 127, 0.3);
  padding: 6px 10px;
}
</style>
