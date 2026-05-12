<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import VChart from 'vue-echarts'
import { NCard, NGrid, NGridItem, NStatistic, NRadioGroup, NRadioButton, NSpin } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as statsApi from '../api/stats'
import type { Overview, Trend } from '../types'

const { t } = useI18n()
const overview = ref<Overview | null>(null)
const trend = ref<Trend | null>(null)
const days = ref<number>(30)
const loading = ref(false)

async function load() {
  loading.value = true
  try {
    const [o, tr] = await Promise.all([statsApi.overview(), statsApi.trend(days.value)])
    overview.value = o
    trend.value = tr
  } finally {
    loading.value = false
  }
}

onMounted(load)
watch(days, load)

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
</script>

<template>
  <NSpin :show="loading">
    <h2 style="margin-top: 0">{{ t('dashboard.overview') }}</h2>

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
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic :label="t('dashboard.adminUsers')" :value="overview?.users.total ?? 0" />
        </NCard>
      </NGridItem>
    </NGrid>

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <h3 style="margin: 0">{{ t('dashboard.trend') }}</h3>
      <NRadioGroup v-model:value="days" size="small">
        <NRadioButton :value="7">{{ t('dashboard.days7') }}</NRadioButton>
        <NRadioButton :value="30">{{ t('dashboard.days30') }}</NRadioButton>
        <NRadioButton :value="90">{{ t('dashboard.days90') }}</NRadioButton>
      </NRadioGroup>
    </div>

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
</template>

<style scoped>
.sub { font-size: 12px; opacity: 0.6; margin-top: 4px; }
</style>
