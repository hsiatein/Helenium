<template>
  <div class="chat-view">
    <div
      ref="contentRef"
      class="chat-content"
    >
      <div
        v-for="(msg, index) in store.messages"
        :key="msg.id"
        :ref="(el) => setTopMessageRef(el as HTMLElement, index)"
        class="message-row"
        :class="{ 'is-user': isUser(msg) }"
      >
        <button class="icon-button delete-button" @click="deleteMessage(msg.id)">
          <img src="/icons/delete_24dp_5985E1_FILL0_wght400_GRAD0_opsz24.png" alt="delete" />
        </button>
        <div class="avatar">
          <img
            v-if="isUser(msg)"
            src="/icons/account_circle_60dp_1F1F1F_FILL0_wght400_GRAD0_opsz48.png"
            alt="User"
          />
          <img v-else src="/icon.png" alt="Heleny" />
        </div>
        <div class="message-content">
          <div class="message-header">{{ headerText(msg) }}</div>
          <div class="message-bubble" :class="{ 'is-user': isUser(msg) }">
            <div v-if="isText(msg)" class="message-text">
              {{ msg.content.Text }}
            </div>
            <div v-else class="message-image">
              <img v-if="imageSrc(msg)" :src="imageSrc(msg)" alt="image" />
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="chat-input">
      <div class="input-wrap">
        <textarea
          v-model="message"
          class="input-area"
          rows="1"
          @keydown="handleKeydown"
        />
      </div>
      <div class="send-wrap">
        <button class="send-button" @click="sendMessage">
          <img src="/icons/send_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.png" alt="send" />
          <span>发送</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch, onMounted } from 'vue';
import { sendCommand } from '../main';
import { store, type ChatMessage } from '../store';

const message = ref('');
const contentRef = ref<HTMLElement | null>(null);
const lastRequestedIdMin = ref<number | null>(null);
const previousFirstMsgId = ref<number | null>(null);
const isInitialLoadDone = ref(false);

let observer: IntersectionObserver | null = null;
const topMessageElement = ref<HTMLElement | null>(null);

const setTopMessageRef = (el: HTMLElement, index: number) => {
  if (index === 0) {
    topMessageElement.value = el;
  }
};

const initObserver = () => {
  observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (!isInitialLoadDone.value) return;

      if (entry.isIntersecting && store.messages.length > 0) {
        const firstMsg = store.messages[0];
        if (firstMsg) {
          const idMin = firstMsg.id;
          if (idMin !== lastRequestedIdMin.value) {
            lastRequestedIdMin.value = idMin;
            sendCommand({ GetHistory: idMin });
          }
        }
      }
    });
  }, {
    root: contentRef.value ?? undefined,
    threshold: 0.1,
  });
};

watch(topMessageElement, (newEl, oldEl) => {
  if (observer) {
    if (oldEl) observer.unobserve(oldEl);
    if (newEl) observer.observe(newEl);
  }
});

onMounted(() => {
  initObserver();

  if (store.messages.length > 0) {
    nextTick(() => {
      const el = contentRef.value;
      if (el) {
        el.scrollTop = el.scrollHeight;
        isInitialLoadDone.value = true;
        const firstMsg = store.messages[0];
        if (firstMsg) previousFirstMsgId.value = firstMsg.id;
      }
    });
  }
});

watch(() => store.messages.length, () => {
  const el = contentRef.value;
  if (!el) return;

  const firstMsg = store.messages[0];
  const currentFirstId = firstMsg ? firstMsg.id : null;

  const oldScrollHeight = el.scrollHeight;
  const oldScrollTop = el.scrollTop;
  const oldClientHeight = el.clientHeight;

  nextTick(() => {
    const newScrollHeight = el.scrollHeight;

    if (!isInitialLoadDone.value) {
      el.scrollTop = newScrollHeight;
      isInitialLoadDone.value = true;
      if (currentFirstId !== null) previousFirstMsgId.value = currentFirstId;
      return;
    }

    const isHistoryPrepend =
      previousFirstMsgId.value !== null &&
      currentFirstId !== null &&
      currentFirstId < previousFirstMsgId.value;

    if (currentFirstId !== null) {
      previousFirstMsgId.value = currentFirstId;
    }

    if (isHistoryPrepend) {
      const heightDiff = newScrollHeight - oldScrollHeight;
      if (heightDiff > 0) {
        el.scrollTop = oldScrollTop + heightDiff;
      }
    } else {
      const wasNearBottom = oldScrollHeight - oldScrollTop - oldClientHeight < 150;
      if (wasNearBottom) {
        el.scrollTop = newScrollHeight;
      }
    }
  });
});

const isUser = (msg: ChatMessage) => msg.role === 'User';

const isText = (msg: ChatMessage) => !!msg.content?.Text;

const headerText = (msg: ChatMessage) => {
  const timeText = formatTime(msg.time);
  if (isUser(msg)) {
    return `${timeText}  User`;
  }
  return `Heleny  ${timeText}`;
};

const formatTime = (raw: string) => {
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

const imageSrc = (msg: ChatMessage) => {
  const base64 = store.images[msg.id];
  const path = msg.content?.Image || '';
  if (!base64) return '';

  const lower = path.toLowerCase();
  let mime = 'image/png';
  if (lower.endsWith('.jpg') || lower.endsWith('.jpeg')) mime = 'image/jpeg';
  if (lower.endsWith('.gif')) mime = 'image/gif';
  if (lower.endsWith('.webp')) mime = 'image/webp';

  return `data:${mime};base64,${base64}`;
};

const sendMessage = () => {
  const msg = message.value.trim();
  if (msg.length === 0) return;
  sendCommand({ UserInput: msg });
  message.value = '';
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key !== 'Enter') return;

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
    return;
  }

  if (event.shiftKey) return;

  event.preventDefault();
  sendMessage();
};

const deleteMessage = (id: number) => {
  sendCommand({ DeleteMemory: { id } });
  const index = store.messages.findIndex((msg) => msg.id === id);
  if (index >= 0) {
    store.messages.splice(index, 1);
  }
  delete store.images[id];
};
</script>

<style scoped>
.chat-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: #f0f8ff;
}

.chat-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.message-row.is-user {
  justify-content: flex-end;
}

.avatar {
  width: 60px;
  height: 60px;
  border-radius: 30px;
  overflow: hidden;
  flex-shrink: 0;
}

.avatar img {
  width: 60px;
  height: 60px;
  display: block;
}

.message-content {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-width: 480px;
}

.message-row.is-user .message-content {
  align-items: flex-end;
}

.message-header {
  font-size: 16px;
  color: #000000;
  white-space: nowrap;
}

.message-bubble {
  background: #7fb5ff;
  border-radius: 12px;
  padding: 12px;
  max-width: 400px;
  word-break: break-word;
}

.message-bubble.is-user {
  background: #b2d4ff;
}

.message-text {
  font-size: 16px;
  color: #000000;
  white-space: pre-wrap;
}

.message-image img {
  width: 100%;
  max-width: 400px;
  max-height: 400px;
  object-fit: contain;
  display: block;
}

.icon-button {
  width: 40px;
  height: 40px;
  border-radius: 20px;
  border: none;
  background: transparent;
  padding: 0;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.icon-button img {
  width: 40px;
  height: 40px;
  display: block;
}

.message-row .avatar {
  order: 1;
}

.message-row .message-content {
  order: 2;
}

.message-row .delete-button {
  order: 3;
}

.message-row.is-user .delete-button {
  order: 1;
}

.message-row.is-user .message-content {
  order: 2;
}

.message-row.is-user .avatar {
  order: 3;
}

.chat-input {
  height: 18%;
  min-height: 120px;
  background: #e2e2e2;
  padding: 12px;
  display: flex;
  gap: 12px;
  align-items: stretch;
}

.input-wrap {
  background: #ffffff;
  border-radius: 16px;
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: stretch;
  padding: 8px 12px;
}

.input-area {
  width: 100%;
  height: 100%;
  border: none;
  resize: none;
  outline: none;
  font-size: 18px;
  background: transparent;
  color: #000000;
  line-height: 1.4;
}

.send-wrap {
  width: 15%;
  min-width: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.send-button {
  width: 100%;
  height: 60%;
  border: none;
  border-radius: 12px;
  background: #7fb5ff;
  color: #000000;
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  cursor: pointer;
}

.send-button img {
  width: 24px;
  height: 24px;
}

@media (max-width: 900px) {
  .chat-content {
    padding: 16px;
  }

  .message-content {
    max-width: 100%;
  }

  .message-bubble,
  .message-image img {
    max-width: 100%;
  }

  .send-wrap {
    width: 30%;
    min-width: 96px;
  }
}
</style>
