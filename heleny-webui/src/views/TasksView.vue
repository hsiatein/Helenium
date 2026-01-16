<template>
  <div class="tasks-view">
    <div class="tasks-title">任务中心</div>
    <div class="tasks-list">
      <div v-if="store.tasks.length === 0" class="tasks-empty">
        暂无任务
      </div>
      <div v-for="task in store.tasks" :key="task.id" class="task-card">
        <div class="task-header" @click="toggleLogs(task)">
          <div class="task-row">
            <div class="task-id-circle">ID</div>
            <div class="task-id">{{ task.id }}</div>
            <div class="task-status">
              <span class="status-pill" :class="statusClass(task.status)">
                {{ statusLabel(task.status) }}
              </span>
            </div>
          </div>
          <div class="task-desc">{{ task.task_description }}</div>
        </div>
        <div class="task-actions">
          <div class="task-spacer" @click="toggleLogs(task)" />
          <button
            v-if="isCancelable(task.status)"
            class="cancel-button"
            @click.stop="cancelTask(task.id)"
          >
            取消任务
          </button>
        </div>
        <div v-if="task.expanded" class="logs-panel">
          <div class="logs-title">任务日志</div>
          <div v-if="task.logs.length === 0" class="logs-empty">
            暂无日志
          </div>
          <div v-for="(log, index) in task.logs" :key="index" class="log-item">
            <span class="log-dot" />
            <span class="log-text">{{ log }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { sendCommand } from '../main';
import { store, type TaskItem } from '../store';

const statusLabel = (status: string) => {
  switch (status) {
    case 'Success':
      return '成功';
    case 'Fail':
      return '失败';
    case 'Running':
      return '运行中';
    case 'Canceled':
      return '已取消';
    case 'Pending':
      return '等待中';
    default:
      return status;
  }
};

const statusClass = (status: string) => {
  switch (status) {
    case 'Success':
      return 'status-success';
    case 'Fail':
      return 'status-fail';
    case 'Running':
      return 'status-running';
    case 'Canceled':
      return 'status-canceled';
    case 'Pending':
      return 'status-pending';
    default:
      return 'status-default';
  }
};

const isCancelable = (status: string) => status === 'Pending' || status === 'Running';

const toggleLogs = (task: TaskItem) => {
  task.expanded = !task.expanded;
  sendCommand({ ToggleTaskLogs: { id: task.id, expanded: task.expanded } });
};

const cancelTask = (id: string) => {
  sendCommand({ CancelTask: { id } });
};
</script>

<style scoped>
.tasks-view {
  height: 100%;
  width: 100%;
  padding: 18px;
  box-sizing: border-box;
  background: #f0f8ff;
}

.tasks-title {
  font-size: 24px;
  text-align: center;
  color: #1c1c1c;
  margin-bottom: 12px;
}

.tasks-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.tasks-empty {
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

.task-card {
  background: #ffffff;
  border-radius: 28px;
  border: 1px solid #dbe6ff;
  box-shadow: 0 2px 8px rgba(211, 229, 255, 0.8);
  overflow: hidden;
}

.task-header {
  padding: 14px 16px 6px;
  cursor: pointer;
}

.task-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.task-id-circle {
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

.task-id {
  flex: 1;
  color: #3f4c67;
  font-size: 16px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-status {
  display: flex;
  justify-content: flex-end;
  min-width: 80px;
}

.status-pill {
  padding: 6px 12px;
  border-radius: 14px;
  font-size: 13px;
  color: #1c1c1c;
  min-width: 64px;
  text-align: center;
}

.status-success {
  background: #b8f2c2;
}

.status-fail {
  background: #ffc7c7;
}

.status-running {
  background: #c5ddff;
}

.status-canceled {
  background: #ffe7a8;
}

.status-pending {
  background: #d7dde3;
}

.status-default {
  background: #d7dde3;
}

.task-desc {
  font-size: 16px;
  color: #1c1c1c;
  margin-top: 8px;
  line-height: 1.4;
}

.task-actions {
  display: flex;
  align-items: center;
  padding: 0 16px 12px;
}

.task-spacer {
  flex: 1;
  height: 1px;
}

.cancel-button {
  height: 36px;
  padding: 0 16px;
  border-radius: 18px;
  border: 1px solid #ff9b9b;
  background: #ffe1e1;
  color: #c62828;
  font-size: 13px;
  cursor: pointer;
}

.cancel-button:hover {
  background: #ffd1d1;
}

.logs-panel {
  margin: 0 16px 16px;
  background: #f6f9ff;
  border-radius: 20px;
  border: 1px solid #dbe6ff;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.logs-title {
  font-size: 14px;
  color: #1c1c1c;
}

.logs-empty {
  font-size: 13px;
  color: #6b6b6b;
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.log-dot {
  width: 6px;
  height: 6px;
  border-radius: 3px;
  background: #7fb5ff;
  margin-top: 6px;
  flex-shrink: 0;
}

.log-text {
  font-size: 13px;
  color: #1c1c1c;
  line-height: 1.4;
  white-space: pre-wrap;
}
</style>
