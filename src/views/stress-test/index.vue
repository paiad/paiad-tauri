<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";

const result = ref("");
const isLoading = ref(false);

const config = ref({
  url: "https://httpbin.org/get",
  method: "GET",
  concurrency: 10,
  total_requests: 100,
});

async function startLoadTest() {
  isLoading.value = true;
  result.value = "⏳ 正在进行压测...";
  try {
    const res = await invoke("run_load_test", { config: config.value });
    console.log("压测的url:", config.value.url);
    result.value = JSON.stringify(res, null, 2);
    ElMessage.success("压测完成");
  } catch (err) {
    result.value = "❌ 压测失败：" + String(err);
    ElMessage.error("压测失败");
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <div class="page-wrapper">
    <el-row :gutter="20" class="content-row">
      <!-- 左侧：表单 -->
      <el-col :xs="24" :sm="12">
        <el-card class="form-card">
          <h1 class="title">📈 Load Test Tool</h1>
          <el-form :model="config" label-position="top">
            <el-form-item label="Request URL">
              <el-input
                  v-model="config.url"
                  placeholder="https://example.com/api"
                  clearable
              />
            </el-form-item>

            <el-form-item label="HTTP Method">
              <el-select v-model="config.method" placeholder="Select method">
                <el-option label="GET" value="GET" />
                <el-option label="POST" value="POST" />
              </el-select>
            </el-form-item>

            <el-form-item label="Concurrency">
              <el-input-number
                  v-model="config.concurrency"
                  :min="1"
                  :step="1"
                  controls-position="right"
              />
            </el-form-item>

            <el-form-item label="Total Requests">
              <el-input-number
                  v-model="config.total_requests"
                  :min="1"
                  :step="1"
                  controls-position="right"
              />
            </el-form-item>

            <el-form-item>
              <el-button
                  type="primary"
                  :loading="isLoading"
                  @click="startLoadTest"
                  :disabled="isLoading"
              >
                {{ isLoading ? "Testing..." : "🚀 Start Load Test" }}
              </el-button>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <!-- 右侧：结果 -->
      <el-col :xs="24" :sm="12">
        <el-card class="result-card" v-if="result">
          <template #header>
            <div class="card-header">
              <span>📄 Result</span>
            </div>
          </template>
          <pre class="result-content">{{ result }}</pre>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped>
@import url("https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap");

.page-wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  min-height: 80vh;
  background: #f9fafb;
  font-family: "Inter", sans-serif;
}

.content-row {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
}

.form-card,
.result-card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  height: 100%;
}

.title {
  font-size: 24px;
  font-weight: 600;
  color: #1f2937;
  text-align: center;
  margin-bottom: 24px;
}

.el-form-item {
  margin-bottom: 20px;
}

.el-input,
.el-select,
.el-input-number {
  width: 100%;
}

.result-content {
  background: #f3f4f6;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #e5e7eb;
  color: #111827;
  font-size: 14px;
  white-space: pre-wrap;
  overflow-x: auto;
  min-height: 200px;
}

.card-header {
  font-weight: 600;
  color: #1f2937;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .page-wrapper {
    padding: 10px;
  }

  .title {
    font-size: 20px;
    margin-bottom: 16px;
  }

  .form-card,
  .result-card {
    margin-bottom: 20px;
  }

  .result-content {
    font-size: 12px;
    min-height: 150px;
  }
}
</style>