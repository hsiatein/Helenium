import { reactive } from 'vue'

export interface ChatMessage {
  id: number;
  role: 'User' | 'Assistant';
  time: string;
  content: {
    Text: string;
  };
}

export const store = reactive({
  totalBusTraffic: [] as [string, number][],
  messages: [] as ChatMessage[],
})
