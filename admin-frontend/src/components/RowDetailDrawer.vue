<script setup lang="ts">
import { computed } from 'vue'
import { NDrawer, NDrawerContent, NCode } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useBreakpoint } from '../composables/useBreakpoint'

defineProps<{ show: boolean; title?: string; data: unknown }>()
defineEmits<{ 'update:show': [v: boolean] }>()
const { t } = useI18n()
const { isMobile } = useBreakpoint()

// Mobile: come up from the bottom at 80% viewport height (easier reach
// than a right-side drawer that gets compressed to a sliver). Desktop:
// keep the established right-side 520px panel.
const placement = computed(() => (isMobile.value ? 'bottom' : 'right'))
const width = computed(() => (isMobile.value ? undefined : 520))
const height = computed(() => (isMobile.value ? '80vh' : undefined))
</script>

<template>
  <NDrawer
    :show="show"
    :placement="placement"
    :width="width"
    :height="height"
    @update:show="$emit('update:show', $event)"
  >
    <NDrawerContent :title="title ?? t('drawer.rowDetail')" closable>
      <NCode language="json" :code="JSON.stringify(data, null, 2)" word-wrap />
    </NDrawerContent>
  </NDrawer>
</template>
