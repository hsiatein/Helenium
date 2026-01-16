import { createRouter, createWebHistory } from 'vue-router'
import ChatView from './views/ChatView.vue'
import ScheduleView from './views/ScheduleView.vue'
import ConsoleView from './views/TerminalView.vue'
import SettingsView from './views/SettingsView.vue'
import TasksView from './views/TasksView.vue'
import ApprovalsView from './views/ApprovalsView.vue'
import ToolsView from './views/ToolsView.vue'

const routes = [
  { path: '/', redirect: '/chat' },
  { path: '/chat', component: ChatView },
  { path: '/schedule', component: ScheduleView },
  { path: '/console', component: ConsoleView },
  { path: '/tasks', component: TasksView },
  { path: '/approvals', component: ApprovalsView },
  { path: '/tools', component: ToolsView },
  { path: '/settings', component: SettingsView },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
