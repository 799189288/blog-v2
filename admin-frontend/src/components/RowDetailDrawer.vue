<script setup lang="ts">
import { NDrawer, NDrawerContent, NCode } from 'naive-ui'

defineProps<{ show: boolean; title?: string; data: unknown }>()
defineEmits<{ 'update:show': [v: boolean] }>()
</script>

<template>
  <NDrawer
    :show="show"
    :width="520"
    placement="right"
    @update:show="$emit('update:show', $event)"
  >
    <NDrawerContent :title="title ?? 'Row detail'" closable>
      <NCode language="json" :code="JSON.stringify(data, null, 2)" word-wrap />
    </NDrawerContent>
  </NDrawer>
</template>
