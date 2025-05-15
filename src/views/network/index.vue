<script setup lang="ts">
import { ref, onUnmounted, watch, computed } from 'vue'
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
  Legend,
  // **新增：导入 ChartOptions 类型**
  type ChartOptions // 使用 type 导入以避免运行时引入
} from 'chart.js'

import * as XLSX from 'xlsx';
import { saveAs } from 'file-saver';

ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend
)

const chartRef = ref<any>(null); // 用于获取 Chart.js 实例和 canvas

const targetUrl = ref(''); // 用户输入的URL

const config = ref({
  window_size: 100, // 窗口大小，仅用于图表和部分计算（如果后端算法需要）
  anomaly_threshold: 2.0,
  sampling_interval: 1000
})

const metrics = ref<AnomalyDetectionResult[]>([]) // metrics 将存储所有数据，不限制长度
const isMonitoring = ref(false)
let monitoringInterval: number | null = null

const urlError = ref(''); // 用于显示URL输入错误信息

const currentPage = ref(1); // 当前页码
const pageSize = ref(10); // 每页显示的条数，这里固定为10


const chartData = ref({
  labels: [] as string[],
  datasets: [
    {
      label: '延迟 (ms)',
      data: [] as number[],
      borderColor: 'rgb(75, 192, 192)',
      backgroundColor: 'rgba(75, 192, 192, 0.5)',
      tension: 0.1,
      fill: false
    },
    {
      label: '丢包率 (%)',
      data: [] as number[],
      borderColor: 'rgb(255, 99, 132)',
      backgroundColor: 'rgba(255, 99, 132, 0.5)',
      tension: 0.1,
      fill: false
    }
  ]
})

const chartOptions: ChartOptions<'line'> = {
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    y: {
      beginAtZero: true,
    }
  },
  plugins: {
    tooltip: {
      mode: 'index', // 现在 TypeScript 知道 'index' 是一个有效的选项
      intersect: false,
    },
    legend: {
      display: true
    },
    title: {
      display: true,
      text: '网络延迟与丢包率趋势'
    }
  },
  hover: {
    mode: 'nearest', // 现在 TypeScript 知道 'nearest' 是一个有效的选项
    intersect: true
  }
}; // 注意这里直接定义了一个常量对象

// **ref 初始化时，直接传入上面定义好的具有明确类型的对象**
const chartOptionsRef = ref(chartOptions);


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

const isValidUrl = (url: string): boolean => {
  if (!url) return false;
  try {
    new URL(url);
    return true;
  } catch (e) {
    return false;
  }
}

const updateMetrics = async () => {
  if (!isMonitoring.value) return

  try {
    const networkMetrics = await invoke<NetworkMetrics>('measure_network_metrics', {
      url: targetUrl.value
    })

    const result = await invoke<AnomalyDetectionResult>('update_metrics', { metrics: networkMetrics })

    // 总是将新结果添加到 metrics 数组，不限制长度
    metrics.value.push(result);
    // 或者 metrics.value = [...metrics.value, result]; // 这种方式对watch更友好

    // 保持图表数据长度限制在 window_size
    const limitSize = config.value.window_size;
    const newLabels = [...chartData.value.labels, new Date(networkMetrics.timestamp).toLocaleTimeString()];
    const newLatencyData = [...chartData.value.datasets[0].data, networkMetrics.latency]
    const newPacketLossData = [...chartData.value.datasets[1].data, networkMetrics.packet_loss]

    if (newLabels.length > limitSize) {
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

    // **修改：当有新数据时，强制将页码设置为第一页，以便显示最新结果**
    currentPage.value = 1;


  } catch (error) {
    console.error('更新指标失败:', error)
    // 发生错误时，如果需要，可以考虑停止监控
    // stopMonitoring();
  }
};


const startMonitoring = async () => {
  if (isMonitoring.value) return

  if (!isValidUrl(targetUrl.value)) {
    urlError.value = "请输入有效的目标地址 (URL)，例如: http://example.com:port/path";
    console.error('启动监控失败: URL 无效')
    return;
  } else {
    urlError.value = '';
  }

  try {
    await invoke('start_network_monitoring', { config: config.value })
    isMonitoring.value = true
    console.log(`开始监控: ${targetUrl.value}`)

    // 立即执行一次更新，并会触发页码回第1页
    await updateMetrics()

    const interval = Math.max(config.value.sampling_interval, 100);
    monitoringInterval = window.setInterval(updateMetrics, interval)

  } catch (error) {
    console.error('启动监控命令失败:', error)
    isMonitoring.value = false
    urlError.value = `启动监控失败: ${error}`;
  }
}

const stopMonitoring = () => {
  console.log('已点击停止监控')
  isMonitoring.value = false
  if (monitoringInterval) {
    clearInterval(monitoringInterval)
    monitoringInterval = null
  }
}

const resetChartAndMetrics = () => {
  console.log('重置图表和数据');
  stopMonitoring();
  metrics.value = []; // 清空表格数据 (全部数据)
  chartData.value = { // 清空图表数据
    labels: [],
    datasets: chartData.value.datasets.map(ds => ({ ...ds, data: [] }))
  };
  currentPage.value = 1; // 重置分页到第一页
  // targetUrl.value = ''; // 可选：重置URL输入框
  // urlError.value = ''; // 可选：清空错误信息
}

const downloadChartImage = () => {
  console.log('下载图表图片 (JPG)');
  if (chartRef.value && chartRef.value.chart) {
    const chartCanvas = chartRef.value.chart.canvas;
    if (chartCanvas) {
      const newCanvas = document.createElement('canvas');
      newCanvas.width = chartCanvas.width;
      newCanvas.height = chartCanvas.height;

      const newCtx = newCanvas.getContext('2d');

      if (newCtx) {
        newCtx.fillStyle = '#FFFFFF';
        newCtx.fillRect(0, 0, newCanvas.width, newCanvas.height);
        newCtx.drawImage(chartCanvas, 0, 0);

        const imageUrl = newCanvas.toDataURL('image/jpeg', 0.9);

        const link = document.createElement('a');
        link.href = imageUrl;
        link.download = `network_metrics_chart_${new Date().toISOString()}.jpg`;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);

        console.log('图表图片下载已触发 (JPG, 白色背景)');

      } else {
        console.error('无法获取新 Canvas 2D 上下文');
      }
    } else {
      console.error('无法获取图表 canvas');
    }
  } else {
    console.error('图表实例未准备好');
  }
}

const downloadMetricsAsExcel = () => {
  console.log('下载全部检测结果为Excel');
  if (metrics.value.length === 0) {
    console.warn('没有数据可供下载');
    alert('没有数据可供下载！');
    return;
  }

  const dataForExcel = metrics.value.map(item => ({
    '时间': new Date(item.metrics.timestamp).toLocaleString(),
    '延迟 (ms)': item.metrics.latency,
    '丢包率 (%)': item.metrics.packet_loss,
    // '带宽': item.metrics.bandwidth,
    '异常分数': item.anomaly_score,
    '是否异常': item.metrics.is_anomaly ? '异常' : '正常'
  }));

  const workbook = XLSX.utils.book_new();
  const worksheet = XLSX.utils.json_to_sheet(dataForExcel);

  XLSX.utils.book_append_sheet(workbook, worksheet, "网络检测结果");

  const excelBuffer = XLSX.write(workbook, { bookType: 'xlsx', type: 'array' });

  const data = new Blob([excelBuffer], { type: 'application/octet-stream' });
  saveAs(data, `network_metrics_result_${new Date().toISOString()}.xlsx`);

  console.log('Excel文件下载已触发');
}


onUnmounted(() => {
  stopMonitoring()
})

watch(targetUrl, (newValue, oldValue) => {
  if (newValue && urlError.value) {
    urlError.value = '';
  }
  if(isMonitoring.value && newValue !== oldValue) {
    console.warn('监控中改变了目标URL，请先停止监控再重新开始。');
  }
});

// **修改：简化 watch(metrics) 逻辑，只处理清空数据的情况**
watch(metrics, (newValue) => {
  if (newValue.length === 0) {
    currentPage.value = 1; // 数据清空时，确保页码回第1页
    // 清空数据时也清空图表
    chartData.value = {
      labels: [],
      datasets: chartData.value.datasets.map(ds => ({ ...ds, data: [] }))
    };
  }
  // 当 metrics 长度增加时，updateMetrics 函数中的 currentPage = 1 会处理页面跳转。
  // 这里不再需要额外的逻辑来跳页或检查超出的页码，因为我们总是回第1页看最新数据。

}, { deep: true });


// 计算属性，用于获取当前页应该显示的数据（已排序和分页）
const paginatedAndReversedMetrics = computed(() => {
  // 1. 复制原始数据
  const data = [...metrics.value];

  // 2. 按时间戳降序排序 (最新的在前面)
  data.sort((a, b) => b.metrics.timestamp - a.metrics.timestamp); // b - a 实现降序

  // 3. 计算当前页数据的起始和结束索引
  const startIndex = (currentPage.value - 1) * pageSize.value;
  const endIndex = startIndex + pageSize.value;

  // 4. 截取当前页的数据
  return data.slice(startIndex, endIndex);
});


// 处理页码变化的函数 (用户手动点击分页时调用)
const handlePageChange = (newPage: number) => {
  console.log(`用户手动切换到页面: ${newPage}`);
  currentPage.value = newPage;
};

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
        <el-form-item label="采样间隔(ms)">
          <el-input-number v-model="config.sampling_interval" :min="100" :max="5000" :step="100" :disabled="isMonitoring" /> </el-form-item>
        <el-form-item label="异常阈值">
          <el-input-number v-model="config.anomaly_threshold" :min="0.1" :max="10" :step="0.1" :disabled="isMonitoring" /> </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="startMonitoring" :disabled="isMonitoring || !targetUrl">
            开始监控
          </el-button>
          <el-button type="danger" @click="stopMonitoring" :disabled="!isMonitoring">
            停止监控
          </el-button>
          <el-button @click="resetChartAndMetrics" :disabled="isMonitoring && metrics.length === 0">
            重置数据
          </el-button>
        </el-form-item>
      </el-form>
    </div>

    <div v-if="urlError" class="url-error-message">
      {{ urlError }} </div>


    <div class="chart-container">
      <Line :data="chartData" :options="chartOptionsRef" ref="chartRef" v-if="chartData.labels.length > 0" />
      <div v-else class="no-data-message">暂无图表数据</div>
    </div>

    <div class="chart-actions" v-if="chartData.labels.length > 0">
      <el-button @click="downloadChartImage">下载图表图片 (JPG)</el-button>
    </div>


    <div class="metrics-table">
      <h3>全部检测结果 (总计 {{ metrics.length }} 条)</h3>
      <el-table :data="paginatedAndReversedMetrics" style="width: 100%">
        <el-table-column prop="metrics.timestamp" label="时间" width="180">
          <template #default="scope">
            {{ new Date(scope.row.metrics.timestamp).toLocaleString() }}
          </template>
        </el-table-column>
        <el-table-column prop="metrics.latency" label="延迟(ms)" /> <el-table-column prop="metrics.packet_loss" label="丢包率(%)" /> <el-table-column prop="anomaly_score" label="异常分数" width="120" />
        <el-table-column prop="metrics.is_anomaly" label="是否异常" width="100">
          <template #default="scope">
            <el-tag :type="scope.row.metrics.is_anomaly ? 'danger' : 'success'">
              {{ scope.row.metrics.is_anomaly ? '异常' : '正常' }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>

      <el-pagination
          v-if="metrics.length > 0"
          :current-page="currentPage"
          :page-size="pageSize"
          :total="metrics.length"
          @current-change="handlePageChange"
          layout="total, prev, pager, next, jumper"
          style="margin-top: 20px; text-align: right;"
      />
      <div v-else class="no-data-message">暂无表格数据</div>


    </div>

    <div class="table-actions" v-if="metrics.length > 0">
      <el-button @click="downloadMetricsAsExcel" type="success">下载全部结果为Excel (.xlsx)</el-button>
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

.url-error-message {
  color: #f56c6c;
  margin-bottom: 15px;
  padding: 10px;
  background-color: #fef0f0;
  border: 1px solid #fcd3d3;
  border-radius: 4px;
  font-size: 0.9em;
}


.chart-container {
  height: 400px;
  margin: 20px 0;
  padding: 20px;
  background-color: white;
  border-radius: 4px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  display: flex;
  justify-content: center;
  align-items: center;
}

.no-data-message {
  color: #909399;
  font-size: 1.2em;
}

.chart-actions,
.table-actions {
  margin-top: 10px;
  margin-bottom: 20px;
  text-align: right;
}


.metrics-table {
  margin-top: 20px;
}

.el-pagination {
  justify-content: flex-end;
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