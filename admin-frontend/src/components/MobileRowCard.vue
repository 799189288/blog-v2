<script setup lang="ts">
// A thin visual container for mobile views that replace a data table.
// Each row of the source data becomes one card with three slots:
//   * title   — bold, larger; the most identifying field (e.g. post title)
//   * body    — secondary metadata; key/value pairs, tags, timestamps
//   * actions — right-aligned button row; wraps when too narrow
//
// We rely on Naive UI CSS variables (--n-color, --n-border-color, etc.)
// so the card adapts to light/dark theme automatically — no theme prop,
// no js theme check.
</script>

<template>
  <div class="m-card">
    <div class="m-card-title">
      <slot name="title" />
    </div>
    <div class="m-card-body">
      <slot name="body" />
    </div>
    <div v-if="$slots.actions" class="m-card-actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<style scoped>
.m-card {
  border: 1px solid var(--n-border-color, rgba(127, 127, 127, 0.2));
  border-radius: 6px;
  padding: 12px 14px;
  margin-bottom: 10px;
  background: var(--n-card-color, transparent);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.m-card-title {
  font-size: 15px;
  font-weight: 600;
  line-height: 1.35;
  word-break: break-word;
}
.m-card-body {
  font-size: 13px;
  opacity: 0.85;
  display: flex;
  flex-direction: column;
  gap: 6px;
  word-break: break-word;
}
.m-card-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: flex-end;
  margin-top: 4px;
}
</style>
