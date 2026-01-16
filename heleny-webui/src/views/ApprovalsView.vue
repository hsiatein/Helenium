<template>
  <div class="approvals-view">
    <div class="approvals-title">审批请求</div>
    <div class="approvals-list">
      <div v-if="store.approvals.length === 0" class="approvals-empty">
        暂无审批请求
      </div>
      <div v-for="req in store.approvals" :key="req.request_id" class="approval-card">
        <div class="approval-body">
          <div class="approval-row">任务ID: {{ req.task_id }}</div>
          <div class="approval-row">任务描述: {{ req.task_description }}</div>
          <div class="approval-row">原因: {{ req.reason }}</div>
          <div class="approval-row">请求描述: {{ req.descripion }}</div>
          <div class="approval-actions">
            <button class="action-button approve" @click="approve(req.request_id)">
              同意
            </button>
            <button class="action-button reject" @click="reject(req.request_id)">
              不同意
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { sendCommand } from '../main';
import { store } from '../store';

const approve = (id: string) => {
  sendCommand({ MakeDecision: { req_id: id, approval: true } });
  store.approvals = store.approvals.filter((item) => item.request_id !== id);
};

const reject = (id: string) => {
  sendCommand({ MakeDecision: { req_id: id, approval: false } });
  store.approvals = store.approvals.filter((item) => item.request_id !== id);
};
</script>

<style scoped>
.approvals-view {
  height: 100%;
  width: 100%;
  padding: 18px;
  box-sizing: border-box;
  background: #f0f8ff;
  overflow-y: auto;
}

.approvals-title {
  font-size: 24px;
  text-align: center;
  color: #1c1c1c;
  margin-bottom: 12px;
}

.approvals-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.approvals-empty {
  height: 120px;
  border-radius: 24px;
  background: #ffffff;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #6b6b6b;
  font-size: 16px;
}

.approval-card {
  background: #ffffff;
  border-radius: 24px;
  border: 1px solid #dbe6ff;
  overflow: hidden;
}

.approval-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.approval-row {
  font-size: 16px;
  color: #1c1c1c;
  line-height: 1.4;
  word-break: break-word;
}

.approval-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 4px;
}

.action-button {
  height: 36px;
  width: 120px;
  border-radius: 18px;
  border: none;
  font-size: 14px;
  color: #1c1c1c;
  cursor: pointer;
}

.action-button.approve {
  background: #7fb5ff;
}

.action-button.approve:hover {
  background: #b2d4ff;
}

.action-button.reject {
  background: #f2a6a6;
}

.action-button.reject:hover {
  background: #f4bcbc;
}
</style>
