<script setup lang="ts">
import { ref, onMounted } from 'vue';

const url = 'https://www.yuketang.cn/v2/web/index';
const iframeRef = ref<HTMLIFrameElement | null>(null);

onMounted(() => {
  if (iframeRef.value) {
    const iframe = iframeRef.value;
    iframe.onload = () => {
      try {
        // 尝试访问 iframe 内的 DOM
        const iframeDoc = iframe.contentDocument || iframe.contentWindow?.document;
        if (iframeDoc) {
          // 查找所有带 target="_blank" 的 <a> 标签
          const links = iframeDoc.querySelectorAll('a[target="_blank"]');
          links.forEach((link) => {
            link.removeAttribute('target'); // 移除 target="_blank"
          });
        }
      } catch (e) {
        console.error('无法访问 iframe 内容，可能存在跨域限制:', e);
      }
    };
  }
});
</script>

<template>
  <div class="iframe-container">
    <iframe ref="iframeRef" :src="url" frameborder="0"></iframe>
  </div>
</template>

<style scoped>
.iframe-container {
  width: 100%;
  height: 88vh;
  overflow: hidden;
}
iframe {
  width: 100%;
  height: 100%;
  border: none;
}
</style>