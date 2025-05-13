<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const result = ref("");
const isLoading = ref(false);

// 默认配置
const config = ref({
  url: "https://httpbin.org/get",
  method: "GET",
  concurrency: 10,
  total_requests: 100,
});

// 发起压测请求
async function startLoadTest() {
  isLoading.value = true;
  result.value = "正在压测，请稍候...";
  try {
    const res = await invoke("run_load_test", { config: config.value });
    console.log("压测的url:" + config.value.url)
    result.value = JSON.stringify(res, null, 2);
  } catch (err) {
    result.value = "压测失败：" + String(err);
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <div class="load-test-container">
    <h1>📈 压测工具</h1>

    <div class="form-group">
      <label>请求地址：</label>
      <input v-model="config.url" type="text" />
    </div>

    <div class="form-group">
      <label>请求方法：</label>
      <select v-model="config.method">
        <option value="GET">GET</option>
        <option value="POST">POST</option>
      </select>
    </div>

    <div class="form-group">
      <label>并发数：</label>
      <input v-model.number="config.concurrency" type="number" min="1" />
    </div>

    <div class="form-group">
      <label>请求总数：</label>
      <input v-model.number="config.total_requests" type="number" min="1" />
    </div>

    <button @click="startLoadTest" :disabled="isLoading">
      {{ isLoading ? "压测中..." : "开始压测" }}
    </button>

    <h2>📄 结果输出：</h2>
    <pre class="result-output">{{ result }}</pre>
  </div>
</template>

<style scoped>
.load-test-container {
  max-width: 600px;
  margin: 40px auto;
  padding: 24px;
  background-color: #ffffff;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  font-family: Arial, sans-serif;
  color: #1e3a8a;
}

h1 {
  text-align: center;
  color: #1d4ed8;
  margin-bottom: 24px;
}

.form-group {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
}

label {
  font-weight: bold;
  margin-bottom: 4px;
}

input,
select {
  padding: 8px;
  border: 1px solid #cbd5e1;
  border-radius: 4px;
}

button {
  background-color: #3b82f6;
  color: white;
  border: none;
  padding: 10px 16px;
  font-size: 16px;
  border-radius: 4px;
  cursor: pointer;
  margin-top: 12px;
  width: 100%;
}

button:disabled {
  background-color: #93c5fd;
  cursor: not-allowed;
}

.result-output {
  background-color: #f3f4f6;
  color: #1e3a8a;
  padding: 12px;
  border-radius: 4px;
  border: 1px solid #d1d5db;
  white-space: pre-wrap;
  margin-top: 16px;
  font-size: 14px;
  overflow-x: auto;
}
</style>
