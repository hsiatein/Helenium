<template>
  <n-layout style="height: 100%">
    <n-layout-content content-style="padding: 24px; overflow-y: auto;">
      <v-chart class="chart" :option="chartOption" autoresize />
    </n-layout-content>
  </n-layout>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { NLayout, NLayoutContent } from 'naive-ui';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { LineChart } from 'echarts/charts';
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
} from 'echarts/components';
import VChart from 'vue-echarts';
import { store } from '../store';

use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
]);

const chartOption = computed(() => ({
  title: {
    text: '总线总流量',
  },
  tooltip: {
    trigger: 'axis',
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '10%',
    containLabel: true,
  },
  xAxis: {
    type: 'time',
    boundaryGap: false,
    animation: false,
  },
  yAxis: {
    type: 'value',
    scale: true,
    min: 0,
  },
  dataZoom: [
    {
      type: 'inside',
      start: 0,
      end: 100,
    },
    {
      start: 0,
      end: 100,
    },
  ],
  series: [
    {
      name: 'Traffic',
      type: 'line',
      showSymbol: false,
      data: store.totalBusTraffic,
      animation: false,
    },
  ],
}));
</script>

<style scoped>
.chart {
  height: 400px;
}
</style>
