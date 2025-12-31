import { createApp } from 'vue'
import App from './App.vue'
import naive from 'naive-ui'
import router from './router'
import { store } from './store'

const app = createApp(App)
app.use(naive)
app.use(router)
app.mount('#app')

const wsPort = import.meta.env.DEV 
  ? '4080' 
  : window.location.port;
export const socket: WebSocket = new WebSocket("ws://127.0.0.1:"+wsPort+"/ws");

socket.onopen = (event: Event) => {
  console.log("✅ 已连接到 Rust 后端");
  socket.send("!get_history 1000000000");
};

socket.onmessage = (event: MessageEvent) => {
  try {
    const data = JSON.parse(event.data);
    if (data.UpdateResource) {
      switch (data.UpdateResource.name) {
        case 'TotalBusTraffic':
          store.totalBusTraffic = data.UpdateResource.payload.TotolBusTraffic;
          break;
        case 'DisplayMessages':
          const newMessages = data.UpdateResource.payload.DisplayMessages;
          if (Array.isArray(newMessages)) {
            // Create a Set of existing IDs for quick lookup
            const existingIds = new Set(store.messages.map(m => m.id));
            
            // Filter out messages that already exist in the store
            const uniqueNewMessages = newMessages.filter(m => !existingIds.has(m.id));
            
            // Add only the new unique messages
            store.messages.push(...uniqueNewMessages);
            
            // Sort the entire array by ID
            store.messages.sort((a, b) => a.id - b.id);
          }
          break;
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

socket.onerror = (error: Event) => {
  console.error("❌ WS 发生错误:", error);
};

socket.onclose = (event: CloseEvent) => {
  console.log(`🔌 连接关闭，代码: ${event.code}, 原因: ${event.reason}`);
};