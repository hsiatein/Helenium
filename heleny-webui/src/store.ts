import { reactive } from 'vue'

export interface ChatMessage {
  id: number;
  role: 'User' | 'Assistant' | 'System';
  time: string;
  content: {
    Text?: string;
    Image?: string;
  }
}

export interface ServiceHealthItem {
  name: string;
  status: string;
}

export interface TaskItem {
  id: string;
  task_description: string;
  status: string;
  logs: string[];
  expanded: boolean;
}

export interface ScheduleItem {
  id: string;
  description: string;
  next_trigger: string;
  triggers: string[];
}

export interface ConsentRequestion {
  request_id: string;
  task_id: string;
  task_description: string;
  reason: string;
  descripion: string;
}

export interface ToolCommand {
  name: string;
  description: string;
}

export interface ToolAbstractItem {
  name: string;
  description: string;
  commands: ToolCommand[];
  available: boolean;
  expanded: boolean;
  desc_expanded: boolean;
  enable: boolean;
}

export const store = reactive({
  totalBusTraffic: [] as [string, number][],
  messages: [] as ChatMessage[],
  images: {} as Record<number, string>,
  servicesHealth: [] as ServiceHealthItem[],
  tasks: [] as TaskItem[],
  schedules: [] as ScheduleItem[],
  approvals: [] as ConsentRequestion[],
  tools: [] as ToolAbstractItem[],
})
