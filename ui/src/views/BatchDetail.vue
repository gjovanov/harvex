<template>
  <div>
    <h1 class="text-h4 mb-4">Batch Detail</h1>
    <p class="text-body-2 text-medium-emphasis">
      Batch ID: {{ $route.params.id }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
const route = useRoute()
</script>
