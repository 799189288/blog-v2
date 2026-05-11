<script setup lang="ts">
import dayjs from 'dayjs'
import type { Comment } from '../types'

defineProps<{ comments: Comment[] }>()
</script>

<template>
  <div v-if="comments.length === 0" class="empty">No comments yet. Be the first!</div>
  <ul v-else class="comment-list">
    <li v-for="c in comments" :key="c.id" class="comment">
      <div class="head">
        <strong>{{ c.author_name }}</strong>
        <span class="date">{{ dayjs(c.created_at).format('YYYY-MM-DD HH:mm') }}</span>
      </div>
      <div class="body">{{ c.content }}</div>
    </li>
  </ul>
</template>

<style scoped>
.empty { opacity: 0.6; font-style: italic; }
.comment-list { list-style: none; padding: 0; margin: 0; }
.comment {
  padding: 12px 0;
  border-bottom: 1px solid rgba(127, 127, 127, 0.15);
}
.comment:last-child {
  border-bottom: none;
}
.head { display: flex; gap: 12px; align-items: baseline; margin-bottom: 6px; }
.date { opacity: 0.6; font-size: 12px; }
.body { white-space: pre-wrap; }
</style>
