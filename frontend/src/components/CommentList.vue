<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Comment } from '../types'
import CommentNode, { type ThreadedComment } from './CommentNode.vue'

const props = defineProps<{ comments: Comment[]; slug: string }>()
const emit = defineEmits<{ submitted: [] }>()

const threaded = computed<ThreadedComment[]>(() => {
  const byId = new Map<number, ThreadedComment>()
  for (const c of props.comments) {
    byId.set(c.id, { ...c, replies: [], replyToName: null })
  }
  const roots: ThreadedComment[] = []
  for (const c of byId.values()) {
    const pid = c.parent_id
    if (pid != null && pid !== c.id && byId.has(pid)) {
      const parent = byId.get(pid)!
      c.replyToName = parent.author_name
      parent.replies.push(c)
    } else {
      roots.push(c)
    }
  }
  return roots
})

const replyTo = ref<number | null>(null)

function openReply(id: number) {
  replyTo.value = replyTo.value === id ? null : id
}

function onReplied() {
  replyTo.value = null
  emit('submitted')
}
</script>

<template>
  <div v-if="threaded.length === 0" class="empty">{{ $t('comments.empty') }}</div>
  <ul v-else class="comment-list">
    <CommentNode
      v-for="c in threaded"
      :key="c.id"
      :node="c"
      :slug="slug"
      :reply-to="replyTo"
      :depth="0"
      @open-reply="openReply"
      @replied="onReplied"
      @cancel-reply="replyTo = null"
    />
  </ul>
</template>

<style scoped>
.empty { opacity: 0.6; font-style: italic; }
.comment-list { list-style: none; padding: 0; margin: 0; }
.comment-list > :deep(.comment) {
  border-bottom: 1px solid rgba(127, 127, 127, 0.15);
}
.comment-list > :deep(.comment):last-child {
  border-bottom: none;
}
</style>
