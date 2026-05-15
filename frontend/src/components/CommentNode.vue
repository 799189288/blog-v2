<script setup lang="ts">
import { NButton } from 'naive-ui'
import dayjs from 'dayjs'
import type { Comment } from '../types'
import CommentForm from './CommentForm.vue'

export interface ThreadedComment extends Comment {
  replies: ThreadedComment[]
  replyToName: string | null
}

const props = defineProps<{
  node: ThreadedComment
  slug: string
  replyTo: number | null
  depth?: number
}>()
const emit = defineEmits<{
  'open-reply': [id: number]
  replied: []
  'cancel-reply': []
}>()
</script>

<template>
  <li class="comment" :class="{ 'is-reply': (depth ?? 0) > 0 }">
    <div class="head">
      <strong>{{ node.author_name }}</strong>
      <span v-if="node.replyToName" class="reply-target">→ @{{ node.replyToName }}</span>
      <span class="date">{{ dayjs(node.created_at).format('YYYY-MM-DD HH:mm') }}</span>
    </div>
    <div class="body">{{ node.content }}</div>
    <div class="row-actions">
      <NButton text size="tiny" @click="emit('open-reply', node.id)">
        {{ replyTo === node.id ? $t('common.cancel') : $t('commentForm.reply') }}
      </NButton>
    </div>

    <div v-if="replyTo === node.id" class="reply-form">
      <CommentForm
        :slug="slug"
        :parent-id="node.id"
        :reply-to-name="node.author_name"
        compact
        @submitted="emit('replied')"
        @cancel="emit('cancel-reply')"
      />
    </div>

    <ul v-if="node.replies.length > 0" class="children">
      <CommentNode
        v-for="child in node.replies"
        :key="child.id"
        :node="child"
        :slug="slug"
        :reply-to="replyTo"
        :depth="(depth ?? 0) + 1"
        @open-reply="(id) => emit('open-reply', id)"
        @replied="emit('replied')"
        @cancel-reply="emit('cancel-reply')"
      />
    </ul>
  </li>
</template>

<style scoped>
.comment {
  padding: 12px 0;
}
.comment.is-reply {
  padding: 10px 0;
}
.head { display: flex; gap: 10px; align-items: baseline; margin-bottom: 6px; flex-wrap: wrap; }
.date { opacity: 0.6; font-size: 12px; }
.reply-target { font-size: 12px; opacity: 0.7; }
.body { white-space: pre-wrap; }
.row-actions { margin-top: 6px; }
.children {
  list-style: none;
  padding: 0 0 0 20px;
  margin: 8px 0 0;
  border-left: 2px solid rgba(127, 127, 127, 0.18);
}
.reply-form {
  margin: 12px 0 4px;
  padding: 12px;
  background: rgba(127, 127, 127, 0.06);
  border-radius: 6px;
}
@media (max-width: 480px) {
  .children { padding-left: 12px; }
  .reply-form { padding: 10px; }
}
</style>
