<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import VChart from 'vue-echarts'
import { NCard, NGrid, NGridItem, NStatistic, NRadioGroup, NRadioButton, NSpin } from 'naive-ui'
import dayjs from 'dayjs'
import * as statsApi from '../api/stats'
import type { Overview, Trend } from '../types'

const overview = ref<Overview | null>(null)
const trend = ref<Trend | null>(null)
const days = ref<number>(30)
const loading = ref(false)

async function load() {
  loading.value = true
  try {
    const [o, t] = await Promise.all([statsApi.overview(), statsApi.trend(days.value)])
    overview.value = o
    trend.value = t
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

const postsOption = computed(() => chartOption('Posts published / day', trend.value?.posts, '#18a058'))
const commentsOption = computed(() => chartOption('Comments / day', trend.value?.comments, '#2080f0'))
</script>

<template>
  <NSpin :show="loading">
    <h2 style="margin-top: 0">Overview</h2>

    <NGrid :x-gap="16" :y-gap="16" :cols="5" responsive="screen" item-responsive>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic label="Posts (total)" :value="overview?.posts.total ?? 0" />
          <div class="sub">{{ overview?.posts.published ?? 0 }} published · {{ overview?.posts.draft ?? 0 }} draft</div>
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic label="Comments" :value="overview?.comments.total ?? 0" />
          <div class="sub">{{ overview?.comments.approved ?? 0 }} approved · {{ overview?.comments.pending ?? 0 }} pending · {{ overview?.comments.spam ?? 0 }} spam</div>
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic label="Pending comments" :value="overview?.comments.pending ?? 0" />
          <div class="sub">need moderation</div>
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic label="Tags" :value="overview?.tags.total ?? 0" />
        </NCard>
      </NGridItem>
      <NGridItem span="5 m:1">
        <NCard>
          <NStatistic label="Admin users" :value="overview?.users.total ?? 0" />
        </NCard>
      </NGridItem>
    </NGrid>

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <h3 style="margin: 0">Trend</h3>
      <NRadioGroup v-model:value="days" size="small">
        <NRadioButton :value="7">7d</NRadioButton>
        <NRadioButton :value="30">30d</NRadioButton>
        <NRadioButton :value="90">90d</NRadioButton>
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
