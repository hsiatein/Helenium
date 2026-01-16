<template>
  <div class="settings-view">
    <div class="settings-title">设置</div>
    <div class="settings-body">
      <div class="settings-card">
        <div class="settings-empty">暂无设置</div>
      </div>
    </div>
    <div class="settings-footer">
      <button class="shutdown-button" @click="shutdown">
        <img src="/icons/power_settings_new_48dp_1F1F1F_FILL0_wght400_GRAD0_opsz48.png" alt="shutdown" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { sendCommand } from '../main';

const shutdown = () => {
  sendCommand('Shutdown');
};
</script>

<style scoped>
.settings-view {
  height: 100%;
  width: 100%;
  padding: 18px;
  box-sizing: border-box;
  background: #f0f8ff;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.settings-title {
  font-size: 24px;
  text-align: center;
  color: #1c1c1c;
}

.settings-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.settings-card {
  background: #ffffff;
  border-radius: 24px;
  border: 1px solid #dbe6ff;
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.settings-empty {
  font-size: 16px;
  color: #6b6b6b;
}

.settings-footer {
  display: flex;
  justify-content: center;
  padding-bottom: 8px;
}

.shutdown-button {
  height: 48px;
  padding: 0 20px;
  border-radius: 24px;
  border: 1px solid #9ec8ff;
  background: #9ec8ff;
  color: #1c1c1c;
  font-size: 16px;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.shutdown-button img {
  width: 24px;
  height: 24px;
}
</style>
