<template>
  <n-layout style="height: 100%">
    <n-layout-content content-style="padding: 24px; overflow-y: auto;">
      <div class="section">
        <div class="section-title">总线总流量</div>
        <v-chart class="chart" :option="chartOption" autoresize />
      </div>
      <div class="section health-section">
        <div class="section-title">服务健康度</div>
        <div class="health-grid">
          <div
            v-for="service in store.servicesHealth"
            :key="service.name"
            class="health-card"
          >
            <span class="status-dot" :class="statusClass(service.status)" />
            <span class="service-name">{{ service.name }}</span>
          </div>
        </div>
      </div>
    </n-layout-content>
  </n-layout>
</template>

<script setup lang="ts">
import { computed } from 'vue';
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

const statusClass = (status: string) => {
  switch (status) {
    case 'Healthy':
      return 'status-healthy';
    case 'Unhealthy':
      return 'status-unhealthy';
    case 'Stopped':
      return 'status-stopped';
    case 'Stopping':
      return 'status-stopping';
    case 'Starting':
      return 'status-starting';
    default:
      return 'status-unknown';
  }
};
</script>

<style scoped>
.section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 20px;
  color: #000000;
  text-align: center;
  margin-bottom: 12px;
}

.chart {
  height: 400px;
}

.health-section {
  padding-bottom: 12px;
}

.health-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}

.health-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: #ffffff;
  border-radius: 20px;
  border: 1px solid #e0e0e0;
}

.status-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  display: inline-block;
  background: #6c757d;
}

.status-healthy {
  background: #28a745;
}

.status-unhealthy {
  background: #ffc107;
}

.status-stopped {
  background: #dc3545;
}

.status-stopping {
  background: #6f42c1;
}

.status-starting {
  background: #007bff;
}

.status-unknown {
  background: #6c757d;
}

.service-name {
  font-size: 16px;
  color: #000000;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
