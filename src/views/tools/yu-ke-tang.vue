<script setup lang="ts">
const url = 'https://www.yuketang.cn/v2/web/index'  // 👈 你要展示的网址
</script>

<template>
  <div class="iframe-container">
    <iframe :src="url" frameborder="0"></iframe>
  </div>
</template>

<style scoped>
.iframe-container {
  width: 100%;
  height: 100vh; /* 占满整个视口 */
  overflow: hidden;
}
iframe {
  width: 100%;
  height: 100%;
  border: none;
}
</style>
