import { createApp } from 'vue'
import App from './App.vue'
import naive from 'naive-ui'
import router from './router'

const app = createApp(App)
app.use(naive)
app.use(router)
app.mount('#app')

const socket: WebSocket = new WebSocket("ws://127.0.0.1:"+window.location.port+"/ws");

socket.onopen = (event: Event) => {
  console.log("✅ 已连接到 Rust 后端");
  socket.send("hello" );
};

socket.onmessage = (event: MessageEvent) => {
  const data = JSON.parse(event.data);
  console.log("📩 收到消息:", data);
};

socket.onerror = (error: Event) => {
  console.error("❌ WS 发生错误:", error);
};

socket.onclose = (event: CloseEvent) => {
  console.log(`🔌 连接关闭，代码: ${event.code}, 原因: ${event.reason}`);
};