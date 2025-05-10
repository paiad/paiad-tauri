<template>
  <div class="dashboard-container">

  </div>
</template>

<script setup lang="ts">
// 组件逻辑
</script>

<style scoped>
.dashboard-container {
  padding: 20px;
}

.data-card {
  height: 180px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: #606266;
}

.card-value {
  font-size: 28px;
  font-weight: bold;
  color: #303133;
  margin: 20px 0;
}

.card-footer {
  display: flex;
  align-items: center;
  color: #909399;
  font-size: 14px;
}

.trend {
  display: flex;
  align-items: center;
  margin-right: 8px;
}

.trend.up {
  color: #67c23a;
}

.trend.down {
  color: #f56c6c;
}

.period {
  color: #909399;
}
</style> 