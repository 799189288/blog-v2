<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import VChart from 'vue-echarts'
import {
  NCard, NGrid, NGridItem, NStatistic, NRadioGroup, NRadioButton, NSpin,
  NList, NListItem, NThing, NTag, NSpace, NEmpty, NButton,
} from 'naive-ui'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as statsApi from '../api/stats'
import type { DashboardResponse, Trend } from '../types'
import { useDictStore } from '../stores/dict'

const { t } = useI18n()
const dict = useDictStore()

const data = ref<DashboardResponse | null>(null)
const trend = ref<Trend | null>(null)
const days = ref<number>(30)
const loading = ref(false)
const loadingTrend = ref(false)

async function loadDashboard() {
  loading.value = true
  try {
    data.value = await statsApi.dashboard()
  } finally {
    loading.value = false
  }
}

async function loadTrend() {
  loadingTrend.value = true
  try {
    trend.value = await statsApi.trend(days.value)
  } finally {
    loadingTrend.value = false
  }
}

onMounted(async () => {
  await dict.ensureAll(['comment.status'])
  await Promise.all([loadDashboard(), loadTrend()])
})
watch(days, loadTrend)

const overview = computed(() => data.value?.overview)
const topPosts = computed(() => data.value?.top_posts ?? [])
const recentComments = computed(() => data.value?.recent_comments ?? [])
const tagCloud = computed(() => data.value?.tag_cloud ?? [])

const pendingCount = computed(() => overview.value?.comments.pending ?? 0)
const draftCount = computed(() => overview.value?.posts.draft ?? 0)
const spamCount = computed(() => overview.value?.comments.spam ?? 0)

// ----- charts -----

function chartOption(title: string, points: { date: string; count: number }[] | undefined, color: string) {
  const xs = (points ?? []).map(p => dayjs(p.date).format('MM-DD'))
  const ys = (points ?? []).map(p => p.count)
  return {
    title: { text: title, left: 'left', textStyle: { fontSize: 14, fontWeight: 500 } },
    grid: { left: 40, right: 20, top: 40, bottom: 30 },
    tooltip: { trigger: 'axis' },
    xAxis: { type: 'category', data: xs, boundaryGap: false },
    yAxis: { type: 'value', minInterval: 1 },
    series: [
      {
        name: title,
        type: 'line',
        smooth: true,
        showSymbol: false,
        areaStyle: { opacity: 0.15 },
        lineStyle: { width: 2 },
        itemStyle: { color },
        data: ys,
      },
    ],
  }
}

const postsOption = computed(() => chartOption(t('dashboard.chartPostsPublished'), trend.value?.posts, '#18a058'))
const commentsOption = computed(() => chartOption(t('dashboard.chartComments'), trend.value?.comments, '#2080f0'))

// Tag-cloud font size: scale by post_count between min..max.
function tagFontSize(count: number): number {
  const counts = tagCloud.value.map(t => t.post_count)
  if (counts.length === 0) return 14
  const min = Math.min(...counts)
  const max = Math.max(...counts)
  if (min === max) return 16
  // 12px..22px
  const t = (count - min) / (max - min)
  return Math.round(12 + t * 10)
}

function commentStatusType(s: string): 'default' | 'success' | 'warning' | 'error' {
  if (s === 'approved') return 'success'
  if (s === 'spam') return 'error'
  return 'warning'
}
</script>

<template>
  <NSpin :show="loading">
    <h2 style="margin-top: 0">{{ t('dashboard.overview') }}</h2>

    <!-- Headline stats -->
    <NGrid :x-gap="16" :y-gap="16" :cols="5" responsive="screen" item-responsive>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic :label="t('dashboard.postsTotal')" :value="overview?.posts.total ?? 0" />
          <div class="sub">{{ t('dashboard.postsBreakdown', { published: overview?.posts.published ?? 0, draft: overview?.posts.draft ?? 0 }) }}</div>
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic :label="t('dashboard.comments')" :value="overview?.comments.total ?? 0" />
          <div class="sub">{{ t('dashboard.commentsBreakdown', { approved: overview?.comments.approved ?? 0, pending: overview?.comments.pending ?? 0, spam: overview?.comments.spam ?? 0 }) }}</div>
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic :label="t('dashboard.pendingComments')" :value="overview?.comments.pending ?? 0" />
          <div class="sub">{{ t('dashboard.needModeration') }}</div>
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic :label="t('dashboard.tags')" :value="overview?.tags.total ?? 0" />
          <div class="sub">&nbsp;</div>
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic :label="t('dashboard.adminUsers')" :value="overview?.users.total ?? 0" />
          <div class="sub">&nbsp;</div>
        </NCard>
      </NGridItem>
    </NGrid>

    <!-- Todos / quick actions -->
    <h3 class="section-title">{{ t('dashboard.todos') }}</h3>
    <NGrid :x-gap="16" :y-gap="16" :cols="3" responsive="screen" item-responsive>
      <NGridItem span="3 m:1">
        <NCard :class="{ 'todo-card': true, 'todo-active': pendingCount > 0 }">
          <div class="todo-label">{{ t('dashboard.todoPending') }}</div>
          <div class="todo-num">{{ pendingCount }}</div>
          <RouterLink :to="{ name: 'manage-comments' }" custom v-slot="{ navigate }">
            <NButton size="small" :disabled="pendingCount === 0" @click="navigate">
              {{ t('dashboard.todoGoModerate') }}
            </NButton>
          </RouterLink>
        </NCard>
      </NGridItem>
      <NGridItem span="3 m:1">
        <NCard :class="{ 'todo-card': true, 'todo-active': draftCount > 0 }">
          <div class="todo-label">{{ t('dashboard.todoDrafts') }}</div>
          <div class="todo-num">{{ draftCount }}</div>
          <RouterLink :to="{ name: 'manage-posts' }" custom v-slot="{ navigate }">
            <NButton size="small" :disabled="draftCount === 0" @click="navigate">
              {{ t('dashboard.todoGoPosts') }}
            </NButton>
          </RouterLink>
        </NCard>
      </NGridItem>
      <NGridItem span="3 m:1">
        <NCard :class="{ 'todo-card': true, 'todo-active': spamCount > 0 }">
          <div class="todo-label">{{ t('dashboard.todoSpam') }}</div>
          <div class="todo-num">{{ spamCount }}</div>
          <RouterLink :to="{ name: 'manage-comments' }" custom v-slot="{ navigate }">
            <NButton size="small" :disabled="spamCount === 0" @click="navigate">
              {{ t('dashboard.todoGoReview') }}
            </NButton>
          </RouterLink>
        </NCard>
      </NGridItem>
    </NGrid>

    <!-- Trend -->
    <div class="section-header">
      <h3 style="margin: 0">{{ t('dashboard.trend') }}</h3>
      <NRadioGroup v-model:value="days" size="small">
        <NRadioButton :value="7">{{ t('dashboard.days7') }}</NRadioButton>
        <NRadioButton :value="30">{{ t('dashboard.days30') }}</NRadioButton>
        <NRadioButton :value="90">{{ t('dashboard.days90') }}</NRadioButton>
      </NRadioGroup>
    </div>
    <NSpin :show="loadingTrend">
      <NGrid :x-gap="16" :y-gap="16" :cols="2" responsive="screen" item-responsive>
        <NGridItem span="2 m:1">
          <NCard>
            <VChart :option="postsOption" autoresize style="height: 280px;" />
          </NCard>
        </NGridItem>
        <NGridItem span="2 m:1">
          <NCard>
            <VChart :option="commentsOption" autoresize style="height: 280px;" />
          </NCard>
        </NGridItem>
      </NGrid>
    </NSpin>

    <!-- Top posts + recent comments -->
    <NGrid :x-gap="16" :y-gap="16" :cols="2" responsive="screen" item-responsive class="bottom-grid">
      <NGridItem span="2 m:1">
        <NCard :title="t('dashboard.topPosts')" style="height: 100%" :content-style="{ display: 'flex', flexDirection: 'column' }">
          <NEmpty v-if="topPosts.length === 0" :description="t('dashboard.noTopPosts')" />
          <ol v-else class="top-list">
            <li v-for="p in topPosts" :key="p.id">
              <RouterLink :to="{ name: 'manage-post-edit', params: { id: p.id } }" class="top-link">
                <span class="top-title" :title="p.title">{{ p.title }}</span>
              </RouterLink>
              <span class="top-meta">
                <span class="meta-chip"><span class="eye">👁</span> {{ p.views }}</span>
                <span class="meta-chip">💬 {{ p.comment_count }}</span>
              </span>
            </li>
          </ol>
        </NCard>
      </NGridItem>
      <NGridItem span="2 m:1">
        <NCard :title="t('dashboard.recentComments')" style="height: 100%" :content-style="{ display: 'flex', flexDirection: 'column' }">
          <NEmpty v-if="recentComments.length === 0" :description="t('dashboard.noRecentComments')" />
          <NList v-else hoverable>
            <NListItem v-for="c in recentComments" :key="c.id">
              <NThing>
                <template #header>
                  <NSpace :size="8" align="center">
                    <strong>{{ c.author_name }}</strong>
                    <NTag size="tiny" :type="commentStatusType(c.status)">
                      {{ dict.label('comment.status', c.status, c.status) }}
                    </NTag>
                    <span class="date">{{ dayjs(c.created_at).format('MM-DD HH:mm') }}</span>
                  </NSpace>
                </template>
                <template #description>
                  <div class="comment-content">{{ c.content }}</div>
                  <div class="comment-on">
                    {{ t('dashboard.onPost') }}
                    <RouterLink :to="{ name: 'manage-post-edit', params: { id: c.post_id } }">
                      {{ c.post_title }}
                    </RouterLink>
                  </div>
                </template>
              </NThing>
            </NListItem>
          </NList>
        </NCard>
      </NGridItem>
    </NGrid>

    <!-- Tag cloud -->
    <NCard :title="t('dashboard.tagCloud')" class="tag-cloud-card">
      <NEmpty v-if="tagCloud.length === 0" :description="t('dashboard.noTags')" />
      <div v-else class="tag-cloud">
        <RouterLink
          v-for="tg in tagCloud"
          :key="tg.id"
          :to="{ name: 'manage-tags' }"
          class="tag-chip"
          :style="{ fontSize: tagFontSize(tg.post_count) + 'px' }"
          :title="t('dashboard.postsCount', { count: tg.post_count })"
        >
          {{ tg.name }}
          <span class="tag-count">·{{ tg.post_count }}</span>
        </RouterLink>
      </div>
    </NCard>
  </NSpin>
</template>

<style scoped>
.sub { font-size: 12px; opacity: 0.6; margin-top: 4px; min-height: 1em; }
.n-grid :deep(.n-grid-item) { display: flex; }
.n-grid :deep(.n-grid-item) > .n-card { flex: 1; }

.section-title { margin: 24px 0 12px; }
.section-header {
  display: flex; justify-content: space-between; align-items: center;
  margin: 24px 0 12px;
  flex-wrap: wrap;
  gap: 8px;
}

.todo-card { position: relative; }
.todo-label { font-size: 12px; opacity: 0.7; }
.todo-num { font-size: 28px; font-weight: 600; margin: 4px 0 10px; }
.todo-active .todo-num { color: #f0a020; }

.top-list { list-style: none; padding: 0; margin: 0; counter-reset: top-rank; }
.top-list li {
  display: flex; align-items: center; gap: 12px;
  padding: 8px 0;
  border-bottom: 1px dashed rgba(127, 127, 127, 0.15);
}
.top-list li:last-child { border-bottom: none; }
.top-list li::before {
  counter-increment: top-rank;
  content: counter(top-rank);
  display: inline-block;
  min-width: 22px;
  height: 22px;
  text-align: center;
  line-height: 22px;
  border-radius: 50%;
  font-size: 12px;
  background: rgba(127, 127, 127, 0.15);
  flex-shrink: 0;
}
.top-list li:nth-child(1)::before { background: #f0a020; color: #fff; }
.top-list li:nth-child(2)::before { background: #888; color: #fff; }
.top-list li:nth-child(3)::before { background: #d29c54; color: #fff; }
.top-link { flex: 1; min-width: 0; text-decoration: none; color: inherit; }
.top-link:hover { text-decoration: underline; }
.top-title {
  display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.top-meta { display: flex; gap: 12px; font-size: 12px; opacity: 0.7; flex-shrink: 0; }
.meta-chip { display: inline-flex; align-items: center; gap: 3px; }

.bottom-grid { margin-top: 16px; }
.bottom-grid :deep(.n-grid-item) { display: flex; }
.bottom-grid :deep(.n-grid-item) > .n-card { flex: 1; }

.date { font-size: 12px; opacity: 0.6; }
.comment-content {
  white-space: pre-wrap; margin: 4px 0;
  display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2;
  -webkit-box-orient: vertical; overflow: hidden;
}
.comment-on { font-size: 12px; opacity: 0.7; }
.comment-on a { color: inherit; }

.tag-cloud-card { margin-top: 16px; }
.tag-cloud { display: flex; flex-wrap: wrap; gap: 8px 14px; align-items: baseline; }
.tag-chip {
  text-decoration: none; color: inherit; line-height: 1.6;
  padding: 2px 4px; border-radius: 4px;
}
.tag-chip:hover { background: rgba(127, 127, 127, 0.12); }
.tag-count { font-size: 0.7em; opacity: 0.5; margin-left: 2px; }
@media (max-width: 768px) {
  .top-list li { flex-wrap: wrap; gap: 8px; }
  .top-meta { flex-basis: 100%; gap: 10px; padding-left: 30px; }
  .top-title { white-space: normal; }
}
</style>
