<script setup lang="ts">
import { computed, ref, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    src: string
    url: string
    title: string
    subtitle?: string
    badge?: string
    accent?: 'cyan' | 'emerald'
  }>(),
  {
    subtitle: '',
    badge: '',
    accent: 'cyan',
  }
)

function resolveAssetUrl(rawSrc: string): string {
  if (
    rawSrc.startsWith('http://') ||
    rawSrc.startsWith('https://') ||
    rawSrc.startsWith('data:') ||
    rawSrc.startsWith('blob:')
  ) {
    return rawSrc
  }
  const base = import.meta.env.BASE_URL || '/'
  const normalizedBase = base.endsWith('/') ? base : `${base}/`
  const cleanPath = rawSrc.replace(/^\.?\//, '')
  return `${normalizedBase}${cleanPath}`
}

const triedSvgFallback = ref(false)
const imageMissing = ref(false)
const currentRawSrc = ref(props.src)

watch(
  () => props.src,
  (next) => {
    currentRawSrc.value = next
    triedSvgFallback.value = false
    imageMissing.value = false
  }
)

const resolvedSrc = computed(() => resolveAssetUrl(currentRawSrc.value))

function onImageError() {
  if (!triedSvgFallback.value && currentRawSrc.value.endsWith('.png')) {
    triedSvgFallback.value = true
    currentRawSrc.value = currentRawSrc.value.replace(/\.png$/, '.svg')
    return
  }
  imageMissing.value = true
}

const displayUrl = computed(() => props.url.replace(/^https?:\/\//, '').replace(/\/$/, ''))
</script>

<template>
  <div
    class="p-5 rounded-xl bg-gray-900/90 border flex flex-col items-center text-center space-y-3"
    :class="
      accent === 'emerald'
        ? 'border-emerald-500/30'
        : 'border-cyan-500/30'
    "
  >
    <div
      v-if="badge"
      class="text-xs font-mono uppercase tracking-wider"
      :class="accent === 'emerald' ? 'text-emerald-400' : 'text-cyan-400'"
    >
      {{ badge }}
    </div>

    <h3 class="text-lg font-bold text-white m-0">{{ title }}</h3>

    <div class="w-52 h-52 rounded-xl bg-white p-1 shadow-lg flex items-center justify-center overflow-hidden">
      <img
        v-if="!imageMissing"
        :src="resolvedSrc"
        :alt="`QR Code for ${title}`"
        class="w-full h-full object-contain"
        @error="onImageError"
      />
      <div
        v-else
        class="w-full h-full rounded-lg border-2 border-dashed border-gray-300 flex flex-col items-center justify-center p-3 text-gray-600 text-xs font-mono"
      >
        <span>Place QR image at</span>
        <code class="mt-1 text-[11px] text-gray-900 break-all">slides/public{{ src }}</code>
      </div>
    </div>

    <a
      :href="url"
      target="_blank"
      rel="noopener noreferrer"
      class="text-sm font-mono font-semibold hover:underline break-all"
      :class="accent === 'emerald' ? 'text-emerald-300' : 'text-cyan-300'"
    >
      {{ displayUrl }}
    </a>

    <p v-if="subtitle" class="text-xs text-gray-400 m-0 max-w-xs">
      {{ subtitle }}
    </p>
  </div>
</template>
