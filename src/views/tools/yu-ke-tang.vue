<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const url = 'https://www.yuketang.cn/v2/web/index'

const openYukeTang = async () => {
  try {
    await invoke('open_url', { url })
  } catch (error) {
    console.error('打开雨课堂失败:', error)
  }
}
</script>

<template>
  <div class="yuketang-container">
    <el-card class="welcome-card">
      <template #header>
        <div class="card-header">
          <span>雨课堂</span>
        </div>
      </template>
      <div class="card-content">
        <p>点击下方按钮打开雨课堂网页</p>
        <el-button type="primary" @click="openYukeTang">
          <el-icon><Link /></el-icon>
          打开雨课堂
        </el-button>
      </div>
    </el-card>
  </div>
</template>

<style scoped>
.yuketang-container {
  padding: 20px;
  height: 100%;
}

.welcome-card {
  max-width: 600px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-content {
  text-align: center;
  padding: 20px 0;
}

.card-content p {
  margin-bottom: 20px;
  color: #606266;
}
</style>
