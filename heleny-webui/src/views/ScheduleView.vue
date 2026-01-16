<template>
  <div class="schedule-view">
    <div class="schedule-title">日程管理</div>
    <div class="schedule-list">
      <div v-if="store.schedules.length === 0" class="schedule-empty">
        暂无日程
      </div>
      <div v-for="schedule in store.schedules" :key="schedule.id" class="schedule-card">
        <div class="schedule-header">
          <div class="schedule-row">
            <div class="schedule-id-circle">ID</div>
            <div class="schedule-id">{{ schedule.id }}</div>
            <div class="schedule-action">
              <button class="cancel-button" @click="cancelSchedule(schedule.id)">
                取消日程
              </button>
            </div>
          </div>
          <div class="schedule-desc">{{ schedule.description }}</div>
          <div class="next-trigger-row">
            <span class="next-label">下一次触发</span>
            <span class="next-trigger">{{ schedule.next_trigger }}</span>
          </div>
          <div class="trigger-title">触发时间</div>
          <div class="trigger-list">
            <span v-for="(trigger, index) in schedule.triggers" :key="index" class="trigger-pill">
              {{ trigger }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { sendCommand } from '../main';
import { store } from '../store';

const cancelSchedule = (id: string) => {
  sendCommand({ CancelSchedule: { id } });
};
</script>

<style scoped>
.schedule-view {
  height: 100%;
  width: 100%;
  padding: 18px;
  box-sizing: border-box;
  background: #f0f8ff;
  overflow-y: auto;
}

.schedule-title {
  font-size: 24px;
  text-align: center;
  color: #1c1c1c;
  margin-bottom: 12px;
}

.schedule-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.schedule-empty {
  height: 120px;
  border-radius: 24px;
  background: #ffffff;
  border: 1px solid #dbe6ff;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #6b6b6b;
  font-size: 16px;
}

.schedule-card {
  background: #ffffff;
  border-radius: 28px;
  border: 1px solid #dbe6ff;
  box-shadow: 0 2px 8px rgba(211, 229, 255, 0.8);
}

.schedule-header {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.schedule-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.schedule-id-circle {
  width: 32px;
  height: 32px;
  border-radius: 16px;
  background: #7fb5ff;
  color: #ffffff;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.schedule-id {
  flex: 1;
  color: #3f4c67;
  font-size: 16px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.schedule-action {
  display: flex;
  justify-content: flex-end;
  min-width: 120px;
}

.cancel-button {
  height: 34px;
  padding: 0 16px;
  border-radius: 17px;
  border: 1px solid #ff9b9b;
  background: #ffe1e1;
  color: #c62828;
  font-size: 13px;
  cursor: pointer;
}

.cancel-button:hover {
  background: #ffd1d1;
}

.schedule-desc {
  font-size: 16px;
  color: #1c1c1c;
  line-height: 1.4;
}

.next-trigger-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.next-label {
  font-size: 16px;
  color: #3f4c67;
}

.next-trigger {
  background: #d9e9ff;
  border: 1px solid #b3d3ff;
  border-radius: 15px;
  height: 30px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  font-size: 13px;
  color: #1c1c1c;
}

.trigger-title {
  font-size: 16px;
  color: #3f4c67;
}

.trigger-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.trigger-pill {
  display: inline-flex;
  align-items: center;
  height: 30px;
  padding: 0 12px;
  border-radius: 15px;
  background: #e8f1ff;
  border: 1px solid #c6dcff;
  font-size: 13px;
  color: #1c1c1c;
  width: fit-content;
}
</style>
