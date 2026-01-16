import { createApp } from 'vue'
import App from './App.vue'
import naive from 'naive-ui'
import router from './router'
import { store } from './store'

const app = createApp(App)
app.use(naive)
app.use(router)
app.mount('#app')

const wsHost = import.meta.env.DEV
  ? `${window.location.hostname}:4080`
  : window.location.host;
const wsProtocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
export const socket: WebSocket = new WebSocket(`${wsProtocol}://${wsHost}/ws`);

type FrontendCommand =
  | { UserInput: string }
  | { GetHistory: number }
  | { GetImage: { id: number; path: string } }
  | { DeleteMemory: { id: number } }
  | { CancelTask: { id: string } }
  | { ToggleTaskLogs: { id: string; expanded: boolean } }
  | { CancelSchedule: { id: string } }
  | { MakeDecision: { req_id: string; approval: boolean } }
  | { EnableTool: { name: string; enable: boolean } }
  | 'GetHealth'
  | 'GetSchedules'
  | 'GetConsentRequestions'
  | 'GetToolAbstrats'
  | 'ReloadTools'
  | 'Shutdown';

export const sendCommand = (command: FrontendCommand) => {
  socket.send(JSON.stringify(command));
};

socket.onopen = (event: Event) => {
  console.log("✅ 已连接到 Rust 后端");
  sendCommand({ GetHistory: 1000000000 });
  sendCommand('GetHealth');
  sendCommand('GetSchedules');
  sendCommand('GetConsentRequestions');
  sendCommand('GetToolAbstrats');
};

socket.onmessage = (event: MessageEvent) => {
  try {
    const data = JSON.parse(event.data);
    if (data.UserDecision?.ConsentRequestions) {
      const requests = data.UserDecision.ConsentRequestions;
      store.approvals = Array.isArray(requests)
        ? requests.map((item: any) => ({
          request_id: String(item.request_id),
          task_id: String(item.task_id),
          task_description: item.task_description ?? '',
          reason: item.reason ?? '',
          descripion: item.descripion ?? '',
        }))
        : [];
      return;
    }

    if (data.UpdateResource) {
      if (data.UpdateResource.payload?.ToolAbstracts) {
        const { abstracts } = data.UpdateResource.payload.ToolAbstracts;
        const existing = new Map(store.tools.map(tool => [tool.name, tool]));
        const tools = Array.isArray(abstracts)
          ? abstracts.map((tool: any) => {
            const name = tool?.name ?? '';
            const prior = existing.get(name);
            const commands = Object.entries(tool?.commands ?? {}).map(([cmdName, cmdDesc]) => ({
              name: String(cmdName),
              description: String(cmdDesc),
            }));
            commands.sort((a, b) => a.name.localeCompare(b.name));
            return {
              name,
              description: tool?.description ?? '',
              commands,
              available: !!tool?.available,
              enable: tool?.enable !== false,
              expanded: prior?.expanded ?? false,
              desc_expanded: prior?.desc_expanded ?? false,
            };
          })
          : [];
        tools.sort((a, b) => {
          if (a.available !== b.available) {
            return a.available ? -1 : 1;
          }
          return a.name.localeCompare(b.name);
        });
        store.tools = tools;
        return;
      }

      if (data.UpdateResource.payload?.Schedules) {
        const schedulePayload = data.UpdateResource.payload.Schedules;
        const schedules = schedulePayload?.schedules ?? {};
        const entries = Object.entries(schedules).map(([id, schedule]) => {
          const nextTrigger = schedule?.next_trigger
            ? formatDateTime(schedule.next_trigger)
            : '没有下次运行';
          const triggers = Array.isArray(schedule?.triggers)
            ? schedule.triggers.map(formatTrigger)
            : [];
          return {
            id,
            description: schedule?.description ?? '',
            next_trigger: nextTrigger,
            triggers,
          };
        });
        entries.sort((a, b) => a.id.localeCompare(b.id));
        store.schedules = entries;
        return;
      }

      if (data.UpdateResource.payload?.TaskAbstract) {
        const { task_abstracts } = data.UpdateResource.payload.TaskAbstract;
        if (Array.isArray(task_abstracts)) {
          const existing = new Map(store.tasks.map(task => [task.id, task]));
          const nextTasks = task_abstracts.map((task) => {
            const id = String(task.id);
            const prev = existing.get(id);
            return {
              id,
              task_description: task.task_description,
              status: task.status,
              logs: prev?.logs ?? [],
              expanded: prev?.expanded ?? false,
            };
          });
          store.tasks = nextTasks;
        }
        return;
      }

      if (data.UpdateResource.payload?.TaskLogs) {
        const { id, logs } = data.UpdateResource.payload.TaskLogs;
        const task = store.tasks.find(item => item.id === String(id));
        if (task && Array.isArray(logs)) {
          task.logs = logs;
        }
        return;
      }

      if (data.UpdateResource.payload?.Health) {
        const health = data.UpdateResource.payload.Health;
        const services = health?.services ?? {};
        const entries = Object.entries(services).map(([name, value]) => {
          const status = Array.isArray(value) ? value[0] : value;
          return { name, status };
        });
        entries.sort((a, b) => a.name.localeCompare(b.name));
        store.servicesHealth = entries;
        return;
      }

      if (data.UpdateResource.payload?.Image) {
        const { id, base64 } = data.UpdateResource.payload.Image;
        store.images[id] = base64;
        return;
      }

      switch (data.UpdateResource.name) {
        case 'TotalBusTraffic':
          store.totalBusTraffic = data.UpdateResource.payload.TotalBusTraffic;
          break;
        case 'DisplayMessages': {
          const payload = data.UpdateResource.payload.DisplayMessages;
          const newMessages = payload?.messages;
          if (Array.isArray(newMessages)) {
            const existingIds = new Set(store.messages.map(m => m.id));
            const uniqueNewMessages = newMessages.filter(m => !existingIds.has(m.id));
            store.messages.push(...uniqueNewMessages);
            store.messages.sort((a, b) => a.id - b.id);

            for (const msg of uniqueNewMessages) {
              const imagePath = msg.content?.Image;
              if (imagePath && store.images[msg.id] === undefined) {
                sendCommand({ GetImage: { id: msg.id, path: imagePath } });
              }
            }
          }
          break;
        }
        default:
          console.log("Unhandled UpdateResource:", data);
      }
    } else {
      console.log("Received other message:", data);
    }
  } catch (error) {
    console.error("Error parsing socket message:", error);
    console.log("Received raw data:", event.data);
  }
};

const formatDateTime = (raw: string) => {
  const date = new Date(raw);
  if (Number.isNaN(date.getTime())) {
    return raw;
  }
  const pad = (value: number) => String(value).padStart(2, '0');
  const year = date.getFullYear();
  const month = pad(date.getMonth() + 1);
  const day = pad(date.getDate());
  const hours = pad(date.getHours());
  const minutes = pad(date.getMinutes());
  const seconds = pad(date.getSeconds());
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
};

const formatTrigger = (trigger: Record<string, any>) => {
  if (trigger?.Once?.time) {
    return `单次 ${formatDateTime(trigger.Once.time)}`;
  }
  if (trigger?.Interval?.interval_minutes !== undefined) {
    const anchor = trigger.Interval.anchor ? formatDateTime(trigger.Interval.anchor) : '';
    return `间隔 ${trigger.Interval.interval_minutes} 分钟, 锚点 ${anchor}`;
  }
  if (trigger?.Daily?.time) {
    return `每日 ${trigger.Daily.time}`;
  }
  if (trigger?.Weekly?.weekday && trigger?.Weekly?.time) {
    const dayMap: Record<string, string> = {
      Mon: '周一',
      Tue: '周二',
      Wed: '周三',
      Thu: '周四',
      Fri: '周五',
      Sat: '周六',
      Sun: '周日',
    };
    const weekday = dayMap[trigger.Weekly.weekday] ?? trigger.Weekly.weekday;
    return `每${weekday} ${trigger.Weekly.time}`;
  }
  if (trigger?.Monthly?.day !== undefined && trigger?.Monthly?.time) {
    return `每月 ${trigger.Monthly.day} 日 ${trigger.Monthly.time}`;
  }
  return JSON.stringify(trigger);
};

socket.onerror = (error: Event) => {
  console.error("❌ WS 发生错误:", error);
};

socket.onclose = (event: CloseEvent) => {
  console.log(`🔌 连接关闭，代码: ${event.code}, 原因: ${event.reason}`);
};
