<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue' // Added watch
import { invoke } from '@tauri-apps/api/core'
import { Line } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
} from 'chart.js'

ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend
)

// **新增：目标地址状态**
const targetUrl = ref(''); // 用户输入的URL

const config = ref({
  window_size: 100,
  anomaly_threshold: 2.0,
  sampling_interval: 1000
})

const metrics = ref<AnomalyDetectionResult[]>([])
const isMonitoring = ref(false)
let monitoringInterval: number | null = null

// **新增：验证状态和错误消息**
const urlError = ref(''); // 用于显示URL输入错误信息

const chartData = ref({
  labels: [] as string[],
  datasets: [
    {
      label: '延迟 (ms)',
      data: [] as number[],
      borderColor: 'rgb(75, 192, 192)',
      tension: 0.1
    },
    {
      label: '丢包率 (%)',
      data: [] as number[],
      borderColor: 'rgb(255, 99, 132)',
      tension: 0.1
    }
  ]
})

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    y: {
      beginAtZero: true
    }
  }
}

interface NetworkMetrics {
  timestamp: number;
  latency: number;
  packet_loss: number;
  bandwidth: number;
  is_anomaly: boolean;
}

interface AnomalyDetectionResult {
  metrics: NetworkMetrics;
  anomaly_score: number;
  threshold: number;
}

// 验证URL是否有效（简单验证）
const isValidUrl = (url: string): boolean => {
  if (!url) return false;
  try {
    // 尝试构造URL对象，会抛出异常如果格式无效
    new URL(url);
    return true;
  } catch (e) {
    return false;
  }
}

// 测量丢包率
const startMonitoring = async () => {
  if (isMonitoring.value) return // 如果已经在监控，则不重复启动

  // **修改：添加URL验证**
  if (!isValidUrl(targetUrl.value)) {
    urlError.value = "请输入有效的目标地址 (URL)，例如: http://example.com:port/path";
    console.error('启动监控失败: URL 无效')
    return; // URL无效，阻止启动
  } else {
    urlError.value = ''; // 清除错误信息
  }


  try {
    // 假设这个命令只需要配置，不需要URL
    await invoke('start_network_monitoring', { config: config.value })
    isMonitoring.value = true
    console.log(`开始监控: ${targetUrl.value}`)


    const updateMetrics = async () => {
      if (!isMonitoring.value) return // 如果已经停止监控，则退出定时器

      try {
        // **修改：传递用户输入的URL给后端命令**
        const networkMetrics = await invoke<NetworkMetrics>('measure_network_metrics', {
          url: targetUrl.value // 使用 targetUrl ref 的当前值
        })

        // 假设 update_metrics 命令接收 metrics 并进行处理
        const result = await invoke<AnomalyDetectionResult>('update_metrics', { metrics: networkMetrics })

        // 更新数据 (保留最近100条)
        const newMetrics = [...metrics.value, result]
        if (newMetrics.length > 100) {
          newMetrics.shift()
        }
        metrics.value = newMetrics

        // 更新图表数据 (保留最近100个点)
        const currentTime = new Date().toLocaleTimeString()
        const newLabels = [...chartData.value.labels, currentTime]
        const newLatencyData = [...chartData.value.datasets[0].data, networkMetrics.latency]
        const newPacketLossData = [...chartData.value.datasets[1].data, networkMetrics.packet_loss]

        if (newLabels.length > 100) {
          newLabels.shift()
          newLatencyData.shift()
          newPacketLossData.shift()
        }

        chartData.value = {
          labels: newLabels,
          datasets: [
            {
              ...chartData.value.datasets[0],
              data: newLatencyData
            },
            {
              ...chartData.value.datasets[1],
              data: newPacketLossData
            }
          ]
        }
      } catch (error) {
        console.error('更新指标失败:', error)
        // 发生错误时，如果需要，可以考虑停止监控
        // stopMonitoring();
      }
    }

    // 立即执行一次
    await updateMetrics()

    // 设置定时器，根据采样间隔调用 updateMetrics
    // 需要确保 sampling_interval >= 100ms 防止频繁调用
    const interval = Math.max(config.value.sampling_interval, 100); // 最小100ms间隔
    monitoringInterval = window.setInterval(updateMetrics, interval)

  } catch (error) {
    console.error('启动监控命令失败:', error)
    isMonitoring.value = false // 启动命令失败时重置监控状态
    // 如果需要，可以显示错误给用户
    // urlError.value = `启动监控失败: ${error}`;
  }
}

const stopMonitoring = () => {
  console.log('已点击停止监控')
  isMonitoring.value = false // 设置监控状态为停止
  if (monitoringInterval) {
    clearInterval(monitoringInterval) // 清除定时器
    monitoringInterval = null
  }
  // 可选：停止后清空图表和数据
  // metrics.value = [];
  // chartData.value = { labels: [], datasets: chartData.value.datasets.map(ds => ({ ...ds, data: [] })) };
}

// **修改：移除 onMounted 时的自动启动**
// onMounted(() => {
//   console.log('已开始停止监控')
//   startMonitoring() // 移除这一行，让用户手动点击开始
// })

onUnmounted(() => {
  // 组件卸载时停止监控，清理定时器
  stopMonitoring()
})

// **新增：监听 targetUrl 变化，清空错误信息**
watch(targetUrl, (newValue) => {
  if (newValue && urlError.value) {
    urlError.value = ''; // 用户开始输入时清空错误信息
  }
  // 如果监控中URL改变，可能需要提示用户或停止当前监控
  if(isMonitoring.value) {
    console.warn('监控中改变了目标URL，请先停止监控再重新开始。');
    // 可以在这里自动停止监控
    // stopMonitoring();
  }
});

</script>

<template>
  <div class="network-monitor">
    <h2>网络异常检测</h2>

    <div class="config-panel">
      <el-form :model="config" label-width="120px">

        <el-form-item label="目标地址 (URL)" :error="urlError" required>
          <el-input
              v-model="targetUrl"
              placeholder="例如: http://example.com:port/path"
              clearable
              :disabled="isMonitoring" /> </el-form-item>


        <el-form-item label="窗口大小">
          <el-input-number v-model="config.window_size" :min="10" :max="1000" :disabled="isMonitoring" /> </el-form-item>
        <el-form-item label="异常阈值">
          <el-input-number v-model="config.anomaly_threshold" :min="0.1" :max="10" :step="0.1" :disabled="isMonitoring" /> </el-form-item>
        <el-form-item label="采样间隔(ms)">
          <el-input-number v-model="config.sampling_interval" :min="100" :max="5000" :step="100" :disabled="isMonitoring" /> </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="startMonitoring" :disabled="isMonitoring || !targetUrl">
            开始监控
          </el-button>
          <el-button type="danger" @click="stopMonitoring" :disabled="!isMonitoring">
            停止监控
          </el-button>
        </el-form-item>
      </el-form>
    </div>

    <div v-if="urlError" class="url-error-message">
      请先输入有效的目标地址！
    </div>


    <div class="chart-container">
      <Line :data="chartData" :options="chartOptions" />
    </div>

    <div class="metrics-table">
      <h3>最近检测结果 ({{ metrics.length }}条)</h3> <el-table :data="metrics.slice(-10)" style="width: 100%">
      <el-table-column prop="metrics.timestamp" label="时间" width="180">
        <template #default="scope">
          {{ new Date(scope.row.metrics.timestamp).toLocaleString() }}
        </template>
      </el-table-column>
      <el-table-column prop="metrics.latency" label="延迟(ms)" width="120" />
      <el-table-column prop="metrics.packet_loss" label="丢包率(%)" width="120" />
      <el-table-column prop="anomaly_score" label="异常分数" width="120" />
      <el-table-column prop="metrics.is_anomaly" label="是否异常" width="120">
        <template #default="scope">
          <el-tag :type="scope.row.metrics.is_anomaly ? 'danger' : 'success'">
            {{ scope.row.metrics.is_anomaly ? '异常' : '正常' }}
          </el-tag>
        </template>
      </el-table-column>
    </el-table>
    </div>
  </div>
</template>

<style scoped>
.network-monitor {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.config-panel {
  margin-bottom: 20px;
  padding: 20px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

/* **新增：URL错误信息样式** */
.url-error-message {
  color: #f56c6c; /* Element UI Danger Red */
  margin-bottom: 15px;
  padding: 10px;
  background-color: #fef0f0; /* Light Red Background */
  border: 1px solid #fcd3d3;
  border-radius: 4px;
}


.chart-container {
  height: 400px;
  margin: 20px 0;
  padding: 20px;
  background-color: white;
  border-radius: 4px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.metrics-table {
  margin-top: 20px;
}

h2 {
  margin-bottom: 20px;
  color: #303133;
}

h3 {
  margin-bottom: 15px;
  color: #606266;
}
</style>