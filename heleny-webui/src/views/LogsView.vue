<template>
  <n-layout style="height: 100%">
    <n-layout-content content-style="padding: 24px; overflow-y: auto;">
      <div>Logs View</div>
      <!-- Add long content for testing scroll -->
      <div v-for="n in 100" :key="n">Item {{ n }}</div>
    </n-layout-content>
  </n-layout>
</template>

<script setup lang="ts">
import { NLayout, NLayoutContent } from 'naive-ui'
</script>
