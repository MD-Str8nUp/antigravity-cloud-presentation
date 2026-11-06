<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    path?: string
  }>(),
  {
    path: 'codelab/index.html',
  }
)

const resolvedHref = computed(() => {
  const base = import.meta.env.BASE_URL || '/'
  const normalizedBase = base.endsWith('/') ? base : `${base}/`
  const cleanPath = props.path.replace(/^\.?\//, '')
  return `${normalizedBase}${cleanPath}`
})
</script>

<template>
  <a :href="resolvedHref" target="_blank" rel="noopener noreferrer">
    <slot>{{ resolvedHref }}</slot>
  </a>
</template>
