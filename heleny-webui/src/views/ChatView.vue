<template>
  <n-layout style="height: 100%; position: relative;">
    <n-layout-content ref="contentRef" content-style="padding: 24px; padding-bottom: 80px; overflow-y: auto;" class="chat-content">
      <div 
        v-for="msg in store.messages" 
        :key="msg.id" 
        class="message-row"
        :class="{ 'is-user': msg.role === 'User' }"
      >
        <div class="avatar">
          <n-avatar v-if="msg.role === 'User'" size="medium" :style="{ backgroundColor: '#2d8cf0' }">
            You
          </n-avatar>
          <n-avatar v-else size="medium" src="/icon.png" />
        </div>
        <div class="message-content">
          <div class="message-header">
            <n-text :depth="2" style="font-weight: bold;">{{ msg.role === 'User' ? 'You' : 'Heleny' }}</n-text>
            <n-text :depth="3" style="font-size: 12px;">{{ new Date(msg.time).toLocaleString() }}</n-text>
          </div>
          <div class="message-bubble">
            <div class="message-body">
              <span style="white-space: pre-wrap;">{{ msg.content.Text }}</span>
            </div>
          </div>
        </div>
      </div>
    </n-layout-content>
    <n-layout-footer bordered style="padding: 12px 24px; position: absolute; bottom: 0; left: 0; right: 0; background-color: #fafafc;">
      <div style="display: flex; align-items: flex-end; gap: 12px;">
        <n-input
          v-model:value="message"
          type="textarea"
          placeholder="发送消息..."
          :autosize="{
            minRows: 1,
            maxRows: 5,
          }"
          style="flex-grow: 1;"
          @keydown.enter="handleEnter"
        />
        <n-button type="primary" @click="sendMessage">发送</n-button>
      </div>
    </n-layout-footer>
  </n-layout>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue';
import { 
  NLayout, NLayoutContent, NLayoutFooter, NInput, NText, NButton, 
  NAvatar 
} from 'naive-ui'
import { socket } from '../main'
import { store } from '../store'

const message = ref('');
const contentRef = ref<InstanceType<typeof NLayoutContent> | null>(null);


const scrollToBottom = () => {
  nextTick(() => {
    const contentEl = contentRef.value?.$el as HTMLElement;
    if (contentEl) {
      contentEl.scrollTop = contentEl.scrollHeight;
    }
  });
};

watch(() => store.messages.length, () => {
  scrollToBottom();
});

const sendMessage = () => {
  let msg=message.value.trim();
  if (msg.length === 0) return;
  socket.send(msg);
  message.value = '';
};

const handleEnter = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    if (event.ctrlKey) {
      event.preventDefault();
      const el = event.target as HTMLTextAreaElement;
      const start = el.selectionStart;
      const end = el.selectionEnd;
      const newValue = message.value.substring(0, start) + '\n' + message.value.substring(end);
      message.value = newValue;
      nextTick(() => {
        el.selectionStart = el.selectionEnd = start + 1;
      });
    } else if (event.shiftKey) {
      return;
    } else {
      event.preventDefault();
      sendMessage();
    }
  }
};
</script>

<style scoped>
.chat-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.message-row {
  display: flex;
  gap: 12px;
}

.message-row.is-user {
  align-self: flex-end;
  flex-direction: row-reverse;
}

.avatar {
  flex-shrink: 0;
}

.message-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-grow: 1; /* Allow content to grow */
  max-width: calc(100% - 60px); /* Adjust based on avatar width + gap */
}

.message-row.is-user .message-header {
  align-self: flex-end;
}

.message-bubble {
  padding: 10px 14px;
  border-radius: 12px;
  background-color: #f0f2f5;
  width: auto; /* Allow width to be determined by content and flex-grow */
  max-width: 100%; /* Ensure it doesn't overflow its parent */
  word-break: break-word; /* Ensure long words break */
}

.message-row.is-user .message-bubble {
  background-color: #cce5ff;
  align-self: flex-end; /* Align the bubble to the end in user messages */
}

.message-header {
  display: flex;
  align-items: center;
  gap: 12px;
}
</style>
