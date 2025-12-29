import { createRouter, createWebHistory } from 'vue-router'
import ChatView from './views/ChatView.vue'
import ScheduleView from './views/ScheduleView.vue'
import ConsoleView from './views/ConsoleView.vue'
import LogsView from './views/LogsView.vue'

const routes = [
  { path: '/', redirect: '/chat' },
  { path: '/chat', component: ChatView },
  { path: '/schedule', component: ScheduleView },
  { path: '/console', component: ConsoleView },
  { path: '/logs', component: LogsView },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
