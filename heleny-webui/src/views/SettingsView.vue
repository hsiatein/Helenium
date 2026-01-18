<template>
  <div class="settings-view">
    <div class="settings-title">设置</div>
    <div class="settings-body">
      <div class="settings-card">
        <div class="settings-actions">
          <button class="settings-item" @click="refreshChat" @pointerdown="onRipple">
            <div class="item-text">
              <div class="item-title">重载聊天配置</div>
              <div class="item-desc">重新加载聊天参数与模型配置</div>
            </div>
            <span class="refresh-button" aria-hidden="true">
              <img src="/icons/refresh_40dp_1F1F1F_FILL0_wght400_GRAD0_opsz40.png" alt="" />
            </span>
          </button>
          <button class="settings-item" @click="refreshSchedule" @pointerdown="onRipple">
            <div class="item-text">
              <div class="item-title">重载日程</div>
              <div class="item-desc">重新加载日程配置文件</div>
            </div>
            <span class="refresh-button" aria-hidden="true">
              <img src="/icons/refresh_40dp_1F1F1F_FILL0_wght400_GRAD0_opsz40.png" alt="" />
            </span>
          </button>
          <button class="settings-item" @click="refreshTools" @pointerdown="onRipple">
            <div class="item-text">
              <div class="item-title">重载工具</div>
              <div class="item-desc">重新加载工具列表</div>
            </div>
            <span class="refresh-button" aria-hidden="true">
              <img src="/icons/refresh_40dp_1F1F1F_FILL0_wght400_GRAD0_opsz40.png" alt="" />
            </span>
          </button>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { sendCommand } from '../main';

const refreshSchedule = () => {
  sendCommand('ReloadSchedule');
};

const refreshTools = () => {
  sendCommand('ReloadTools');
};

const refreshChat = () => {
  sendCommand('ReloadChat');
};

const onRipple = (event: PointerEvent) => {
  const target = event.currentTarget as HTMLElement | null;
  if (!target) return;
  const rect = target.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  target.style.setProperty('--ripple-x', `${x}px`);
  target.style.setProperty('--ripple-y', `${y}px`);
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
  min-height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.settings-actions {
  width: 100%;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-sizing: border-box;
}

.settings-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 16px;
  border-radius: 18px;
  background: #f6f9ff;
  border: 1px solid #dbe6ff;
  cursor: pointer;
  width: 100%;
  text-align: left;
  position: relative;
  overflow: hidden;
  transition: transform 0.12s ease, box-shadow 0.12s ease, background 0.12s ease;
}

.settings-item::after {
  content: '';
  position: absolute;
  inset: 0;
  background: radial-gradient(circle at var(--ripple-x, 50%) var(--ripple-y, 50%), rgba(127, 181, 255, 0.35), transparent 60%);
  opacity: 0;
  transition: opacity 0.25s ease;
  pointer-events: none;
}

.settings-item:hover {
  background: #eef4ff;
  box-shadow: 0 4px 10px rgba(211, 229, 255, 0.8);
}

.settings-item:active {
  transform: scale(0.99);
}

.settings-item:active::after {
  opacity: 1;
}

.settings-item:focus-visible {
  outline: 2px solid #7fb5ff;
  outline-offset: 2px;
}

.item-text {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.item-title {
  font-size: 16px;
  color: #1c1c1c;
}

.item-desc {
  font-size: 13px;
  color: #6b6b6b;
}

.refresh-button {
  width: 44px;
  height: 44px;
  border-radius: 22px;
  background: #e8f1ff;
  border: 1px solid #c6dcff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
}

.refresh-button img {
  width: 22px;
  height: 22px;
}


</style>
