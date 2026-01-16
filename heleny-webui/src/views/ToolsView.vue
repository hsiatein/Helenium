<template>
  <div class="tools-view">
    <div class="tools-title">工具中心</div>
    <div class="tools-list">
      <div v-if="store.tools.length === 0" class="tools-empty">
        暂无工具
      </div>
      <div v-for="tool in store.tools" :key="tool.name" class="tool-card">
        <div class="tool-header" @click="toggleExpanded(tool)">
          <div class="tool-base">
            <button class="status-toggle" @click.stop="toggleEnable(tool)">
              <span class="status-dot" :class="statusClass(tool)" />
            </button>
            <span class="name-pill">{{ tool.name }}</span>
          </div>
          <div class="tool-desc" :class="{ expanded: tool.desc_expanded }">
            {{ tool.description }}
          </div>
          <button class="desc-toggle" @click.stop="toggleDesc(tool)">
            {{ tool.desc_expanded ? '收起' : '展开' }}
          </button>
        </div>
        <div v-if="tool.expanded" class="commands-panel">
          <div class="commands-title">命令列表</div>
          <div v-if="tool.commands.length === 0" class="commands-empty">暂无命令</div>
          <div v-for="cmd in tool.commands" :key="cmd.name" class="command-item">
            <span class="command-pill">{{ cmd.name }}</span>
            <div class="command-desc">{{ cmd.description }}</div>
          </div>
        </div>
      </div>
    </div>
    <button class="refresh-button" @click="refreshTools">
      <img src="/icons/refresh_40dp_1F1F1F_FILL0_wght400_GRAD0_opsz40.png" alt="refresh" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { sendCommand } from '../main';
import { store, type ToolAbstractItem } from '../store';

const toggleExpanded = (tool: ToolAbstractItem) => {
  tool.expanded = !tool.expanded;
};

const toggleDesc = (tool: ToolAbstractItem) => {
  tool.desc_expanded = !tool.desc_expanded;
};

const toggleEnable = (tool: ToolAbstractItem) => {
  const next = !tool.enable;
  tool.enable = next;
  sendCommand({ EnableTool: { name: tool.name, enable: next } });
};

const refreshTools = () => {
  sendCommand('ReloadTools');
};

const statusClass = (tool: ToolAbstractItem) => {
  if (!tool.available) return 'status-unavailable';
  return tool.enable ? 'status-enabled' : 'status-disabled';
};
</script>

<style scoped>
.tools-view {
  height: 100%;
  width: 100%;
  padding: 18px;
  box-sizing: border-box;
  background: #f0f8ff;
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
}

.tools-title {
  font-size: 24px;
  text-align: center;
  color: #1c1c1c;
  margin-bottom: 12px;
}

.tools-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
  flex: 0 0 auto;
  padding-bottom: 80px;
}

.tools-empty {
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

.tool-card {
  background: #ffffff;
  border-radius: 28px;
  border: 1px solid #dbe6ff;
  box-shadow: 0 2px 8px rgba(211, 229, 255, 0.8);
  overflow: hidden;
}

.tool-header {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  cursor: pointer;
}

.tool-base {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-toggle {
  width: 30px;
  height: 30px;
  border: none;
  background: transparent;
  padding: 0;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 5px;
  display: inline-block;
}

.status-enabled {
  background: #39d98a;
}

.status-disabled {
  background: #f4c542;
}

.status-unavailable {
  background: #ff5b5b;
}

.name-pill {
  display: inline-flex;
  align-items: center;
  padding: 0 12px;
  height: 32px;
  border-radius: 16px;
  background: #cfe2ff;
  border: 1px solid #9ec8ff;
  font-size: 16px;
  color: #1c1c1c;
}

.tool-desc {
  font-size: 16px;
  color: #000000;
  line-height: 1.4;
  max-height: 132px;
  overflow: hidden;
}

.tool-desc.expanded {
  max-height: none;
}

.desc-toggle {
  height: 32px;
  border-radius: 16px;
  border: 1px solid #dbe6ff;
  background: #eef4ff;
  width: fit-content;
  padding: 0 16px;
  font-size: 16px;
  color: #3f4c67;
  cursor: pointer;
}

.commands-panel {
  background: #f6f9ff;
  border-radius: 20px;
  border: 1px solid #dbe6ff;
  margin: 0 12px 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.commands-title {
  font-size: 14px;
  color: #1c1c1c;
}

.commands-empty {
  font-size: 13px;
  color: #6b6b6b;
}

.command-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.command-pill {
  height: 26px;
  border-radius: 13px;
  background: #dbe6ff;
  border: 1px solid #b6ceff;
  padding: 0 12px;
  width: fit-content;
  display: inline-flex;
  align-items: center;
  font-size: 12px;
  color: #1c1c1c;
}

.command-desc {
  font-size: 13px;
  color: #1c1c1c;
  line-height: 1.4;
}

.refresh-button {
  position: fixed;
  bottom: 24px;
  right: 24px;
  width: 52px;
  height: 52px;
  border-radius: 26px;
  background: #e8f1ff;
  border: 1px solid #c6dcff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  margin-left: auto;
}

.refresh-button img {
  width: 28px;
  height: 28px;
}
</style>
