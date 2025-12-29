<script setup lang="ts">
import { h, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import type { GlobalThemeOverrides } from 'naive-ui'
import { NConfigProvider, NLayout, NLayoutSider, NMenu, NIcon, NText, NAvatar } from 'naive-ui'
import { ChatboxEllipsesOutline, CalendarOutline, TerminalOutline, DocumentTextOutline } from '@vicons/ionicons5'

const themeOverrides: GlobalThemeOverrides = {
  common: {
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
    label: () => h(RouterLink, { to: '/console' }, { default: () => '控制台' }),
    key: 'console',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(TerminalOutline) }),
  },
  {
    label: () => h(RouterLink, { to: '/logs' }, { default: () => '日志' }),
    key: 'logs',
    icon: () => h(NIcon, { class: 'menu-icon', style: { fontSize: '22px' } }, { default: () => h(DocumentTextOutline) }),
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
      </n-layout-sider>
      <n-layout style="height: 100%">
        <router-view />
      </n-layout>
    </n-layout>
  </n-config-provider>
</template>

<style scoped>
.sidebar-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  font-size: 20px;
  font-weight: 600;
  transition: padding 0.3s, justify-content 0.3s;
}
.sidebar-header.collapsed {
  padding: 16px 8px;
}
.sidebar-header :deep(.n-avatar) {
  flex-shrink: 0;
}
.sidebar-title {
  color: #08315c;
}
:deep(.menu-icon) {
  flex-shrink: 0;
}
</style>
