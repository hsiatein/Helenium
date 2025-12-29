<template>
  <n-layout style="height: 100%; position: relative;">
    <n-layout-content content-style="padding: 24px; padding-bottom: 80px; overflow-y: auto;">
      <n-text>Chat content will go here.</n-text>
    </n-layout-content>
    <n-layout-footer bordered style="padding: 12px 24px; position: absolute; bottom: 0; left: 0; right: 0; background-color: #fafafc;">
      <div style="display: flex; gap: 12px;">
        <n-input placeholder="发送消息..." style="flex-grow: 1;" />
        <n-button type="primary">发送</n-button>
      </div>
    </n-layout-footer>
  </n-layout>
</template>

<script setup lang="ts">
import { NLayout, NLayoutContent, NLayoutFooter, NInput, NText, NButton } from 'naive-ui'
</script>
