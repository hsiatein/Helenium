<script setup lang="ts">
import { h, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import type { GlobalThemeOverrides } from 'naive-ui'
import { NConfigProvider, NLayout, NLayoutSider, NMenu, NIcon, NText, NAvatar } from 'naive-ui'
import { ChatboxEllipsesOutline, CalendarOutline, TerminalOutline, SettingsOutline, ListOutline, CheckmarkCircleOutline, ConstructOutline, PowerOutline } from '@vicons/ionicons5'
import { sendCommand } from './main'

const themeOverrides: GlobalThemeOverrides = {
  common: {
    borderRadius: '10px',
    primaryColor: '#1f6feb',
    primaryColorHover: '#1456c3',
    primaryColorPressed: '#1f6feb',
    bodyColor: '#f0f8ff',
    cardColor: '#ffffff',
    textColorBase: '#1f2b3a',
  },
  Layout: {
    siderColor: '#5eb5ff',
    headerBorderColor: '#c7dbf4',
    footerBorderColor: '#c7dbf4',
  },
  Menu: {
    itemTextColor: '#08315c',
    itemIconColor: '#08315c',
    itemTextColorHover: '#08315c',
    itemIconColorHover: '#08315c',
    itemColorActive: 'rgba(255, 255, 255, 0.35)',
    itemColorHover: 'rgba(255, 255, 255, 0.2)',
  },
}

const menuOptions = [
  {
    label: () => h(RouterLink, { to: '/chat' }, { default: () => '聊天' }),
    key: 'chat',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(ChatboxEllipsesOutline) }),
  },
  {
    label: () => h(RouterLink, { to: '/schedule' }, { default: () => '日程' }),
    key: 'schedule',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(CalendarOutline) }),
  },
  {
    label: () => h(RouterLink, { to: '/console' }, { default: () => '终端' }),
    key: 'console',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(TerminalOutline) }),
  },
  {
    label: () => h(RouterLink, { to: '/tasks' }, { default: () => '任务' }),
    key: 'tasks',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(ListOutline) }),
  },
  {
    label: () => h(RouterLink, { to: '/approvals' }, { default: () => '审批' }),
    key: 'approvals',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(CheckmarkCircleOutline) }),
  },
  {
    label: () => h(RouterLink, { to: '/tools' }, { default: () => '工具' }),
    key: 'tools',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(ConstructOutline) }),
  },
  {
    label: () => h(RouterLink, { to: '/settings' }, { default: () => '设置' }),
    key: 'settings',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(SettingsOutline) }),
  },
]

const collapsed = ref(false)
const route = useRoute()
const activeKey = ref(String(route.name))

watch(
  () => route.name,
  () => {
    activeKey.value = String(route.name)
  }
)

const shutdown = () => {
  sendCommand('Shutdown')
}
</script>

<template>
  <n-config-provider :theme-overrides="themeOverrides">
    <n-layout has-sider style="height: 100vh">
      <n-layout-sider
        bordered
        collapse-mode="width"
        :collapsed-width="64"
        :width="240"
        :collapsed="collapsed"
        show-trigger
        @collapse="collapsed = true"
        @expand="collapsed = false"
      >
        <div class="sidebar-content">
        <div class="sidebar-header" :class="{ 'collapsed': collapsed }">
          <n-avatar :size="48" src="/icon.png" />
          <n-text v-if="!collapsed" class="sidebar-title">Heleny</n-text>
        </div>
        <n-menu
          v-model:value="activeKey"
          :collapsed="collapsed"
          :collapsed-width="64"
          :collapsed-icon-size="22"
          :options="menuOptions"
        />
        <div class="sidebar-footer">
          <button class="shutdown-button" @click="shutdown">
            <n-icon class="shutdown-icon">
              <PowerOutline />
            </n-icon>
            <span v-if="!collapsed" class="shutdown-label">关闭</span>
          </button>
        </div>
        </div>
      </n-layout-sider>
      <n-layout style="height: 100%">
        <div class="content-wrap">
          <router-view />
        </div>
      </n-layout>
    </n-layout>
  </n-config-provider>
</template>

<style scoped>
.content-wrap {
  height: 100%;
  min-height: 0;
}
.sidebar-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.sidebar-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  font-size: 20px;
  font-weight: 600;
  transition: padding 0.3s, justify-content 0.3s;
}
.sidebar-footer {
  margin-top: auto;
  padding: 16px;
  display: flex;
  justify-content: center;
}
.shutdown-button {
  width: 100%;
  height: 48px;
  border-radius: 24px;
  border: 1px solid #9ec8ff;
  background: #9ec8ff;
  color: #1c1c1c;
  font-size: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  cursor: pointer;
}
.shutdown-icon {
  font-size: 22px;
}
.shutdown-label {
  font-size: 16px;
}
.sidebar-header.collapsed {
  padding: 16px 8px;
}
.sidebar-header :deep(.n-avatar) {
  flex-shrink: 0;
}
.sidebar-title {
  color: aliceblue;
  font-size: 36px;
  font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
}
:deep(.menu-icon) {
  flex-shrink: 0;
}
</style>
