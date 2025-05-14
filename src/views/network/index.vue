<script setup lang="ts">
import { ref, onUnmounted, watch, computed } from 'vue' // 新增 computed
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

// **新增：引入 xlsx 和 file-saver**
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

// **新增：图表 canvas 的引用**
const chartRef = ref<any>(null); // 用于获取 Chart.js 实例和 canvas

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

// **新增：分页状态**
const currentPage = ref(1); // 当前页码
const pageSize = ref(10); // 每页显示的条数，这里固定为10


const chartData = ref({
  labels: [] as string[],
  datasets: [
    {
      label: '延迟 (ms)',
      data: [] as number[],
      borderColor: 'rgb(75, 192, 192)',
      backgroundColor: 'rgba(75, 192, 192, 0.5)', // 添加背景色让图表更明显
      tension: 0.1,
      fill: false // 不填充区域
    },
    {
      label: '丢包率 (%)',
      data: [] as number[],
      borderColor: 'rgb(255, 99, 132)',
      backgroundColor: 'rgba(255, 99, 132, 0.5)', // 添加背景色
      tension: 0.1,
      fill: false // 不填充区域
    }
  ]
})

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    y: {
      beginAtZero: true,
      // 你当前的配置没有禁用 grid line，所以它们会被绘制并包含在图片中
      // 如果需要显式控制 grid line，可以这样设置：
      // grid: {
      //   display: true, // 是否显示网格线
      //   drawOnChartArea: true, // 是否在图表区域绘制网格线
      //   drawTicks: true, // 是否绘制刻度旁边的线
      //   color: 'rgba(0, 0, 0, 0.1)', // 网格线颜色
      // }
    }
  },
  plugins: { // 添加插件配置
    tooltip: { // 工具提示
      mode: 'index',
      intersect: false,
    },
    legend: { // 图例
      display: true
    },
    title: { // 标题
      display: true,
      text: '网络延迟与丢包率趋势'
    }
  },
  hover: { // 悬停效果
    mode: 'nearest',
    intersect: true
  }
}

interface NetworkMetrics {
  timestamp: number;
  latency: number;
  packet_loss: number;
  bandwidth: number; // 虽然图表没用带宽，但数据结构里有
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

// 测量网络指标并更新图表和表格
const updateMetrics = async () => {
  if (!isMonitoring.value) return // 如果已经停止监控，则退出定时器

  try {
    // 传递用户输入的URL给后端命令
    const networkMetrics = await invoke<NetworkMetrics>('measure_network_metrics', {
      url: targetUrl.value // 使用 targetUrl ref 的当前值
    })

    // 假设 update_metrics 命令接收 metrics 并进行处理，返回包含异常信息的结构
    const result = await invoke<AnomalyDetectionResult>('update_metrics', { metrics: networkMetrics })

    // 更新数据 (保留最近100条，根据 config.window_size)
    // **修改：这里的长度限制应该使用 config.value.window_size**
    const limitSize = config.value.window_size;
    const newMetrics = [...metrics.value, result];
    if (newMetrics.length > limitSize) {
      newMetrics.shift(); // 移除最旧的数据
    }
    metrics.value = newMetrics;

    // **新增：有新数据时重置回第一页，以便用户看到最新数据**
    // 但是如果用户正在看后面的页码，突然跳回去可能不好，可以根据需求调整
    // 这里简单实现有新数据就跳回第一页
    currentPage.value = 1;


    // 更新图表数据 (保留最近100个点，与metrics同步长度)
    // **修改：这里的长度限制也应与metrics同步**
    const newLabels = [...chartData.value.labels, new Date(networkMetrics.timestamp).toLocaleTimeString()]; // 使用后端返回的时间戳
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
  } catch (error) {
    console.error('更新指标失败:', error)
    // 发生错误时，如果需要，可以考虑停止监控
    // stopMonitoring();
  }
};


// 启动监控
const startMonitoring = async () => {
  if (isMonitoring.value) return // 如果已经在监控，则不重复启动

  // 添加URL验证
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

    // 立即执行一次更新
    await updateMetrics()

    // 设置定时器，根据采样间隔调用 updateMetrics
    const interval = Math.max(config.value.sampling_interval, 100); // 最小100ms间隔
    monitoringInterval = window.setInterval(updateMetrics, interval)

  } catch (error) {
    console.error('启动监控命令失败:', error)
    isMonitoring.value = false // 启动命令失败时重置监控状态
    // 如果需要，可以显示错误给用户
    urlError.value = `启动监控失败: ${error}`; // 在URL错误处显示启动失败信息
  }
}

// 停止监控
const stopMonitoring = () => {
  console.log('已点击停止监控')
  isMonitoring.value = false // 设置监控状态为停止
  if (monitoringInterval) {
    clearInterval(monitoringInterval) // 清除定时器
    monitoringInterval = null
  }
}

// **新增：重置图表、数据和分页**
const resetChartAndMetrics = () => {
  console.log('重置图表和数据');
  stopMonitoring(); // 重置时先停止监控
  metrics.value = []; // 清空表格数据
  chartData.value = { // 清空图表数据
    labels: [],
    datasets: chartData.value.datasets.map(ds => ({ ...ds, data: [] }))
  };
  currentPage.value = 1; // **新增：重置分页到第一页**
  // 可选：重置URL输入框
  // targetUrl.value = '';
  // urlError.value = '';
}

// 修改：下载图表图片为 JPG，使用临时 Canvas 添加白色背景
const downloadChartImage = () => {
  console.log('下载图表图片 (JPG)');
  if (chartRef.value && chartRef.value.chart) {
    const chartCanvas = chartRef.value.chart.canvas;
    if (chartCanvas) {
      // **创建新的临时 Canvas 元素**
      const newCanvas = document.createElement('canvas');
      newCanvas.width = chartCanvas.width; // 设置与原图表 Canvas 相同的宽度
      newCanvas.height = chartCanvas.height; // 设置与原图表 Canvas 相同的高度

      // **获取临时 Canvas 的 2D 上下文**
      const newCtx = newCanvas.getContext('2d');

      if (newCtx) {
        // **在临时 Canvas 上绘制白色背景**
        newCtx.fillStyle = '#FFFFFF'; // 设置背景颜色为白色
        newCtx.fillRect(0, 0, newCanvas.width, newCanvas.height); // 绘制白色矩形覆盖整个 Canvas

        // **将原图表 Canvas 的内容绘制到临时 Canvas 上**
        // 这会将 Chart.js 绘制的图表元素绘制在刚刚创建的白色背景之上
        newCtx.drawImage(chartCanvas, 0, 0);

        // **从临时 Canvas 获取图片数据 URL**
        const imageUrl = newCanvas.toDataURL('image/jpeg', 0.9); // 导出为 JPG 格式


        // **创建并触发下载链接 (与之前相同)**
        const link = document.createElement('a');
        link.href = imageUrl;
        link.download = `network_metrics_chart_${new Date().toISOString()}.jpg`; // 设置下载文件名
        document.body.appendChild(link); // 必须添加到 DOM 才能点击
        link.click(); // 模拟点击下载
        document.body.removeChild(link); // 下载后移除临时链接

        console.log('图表图片下载已触发 (JPG, 白色背景)');

        // **注意：不需要在这里恢复原 Canvas 的状态，因为我们没有修改原 Canvas**

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

// **新增：下载检测结果为Excel (导出metrics中的全部数据)**
const downloadMetricsAsExcel = () => {
  console.log('下载全部检测结果为Excel');
  if (metrics.value.length === 0) {
    console.warn('没有数据可供下载');
    alert('没有数据可供下载！'); // 提示用户没有数据
    return;
  }

  // **修改：准备要导出到 Excel 的数据 - 导出metrics中的全部数据**
  const dataForExcel = metrics.value.map(item => ({
    '时间': new Date(item.metrics.timestamp).toLocaleString(),
    '延迟 (ms)': item.metrics.latency,
    '丢包率 (%)': item.metrics.packet_loss,
    // '带宽': item.metrics.bandwidth, // 包含带宽数据
    '异常分数': item.anomaly_score,
    '是否异常': item.metrics.is_anomaly ? '异常' : '正常' // 将布尔值转为文字
  }));

  // 创建工作簿和工作表
  const workbook = XLSX.utils.book_new();
  const worksheet = XLSX.utils.json_to_sheet(dataForExcel);

  // 将工作表添加到工作簿
  XLSX.utils.book_append_sheet(workbook, worksheet, "网络检测结果"); // Sheet name

  // 生成 XLSX 文件
  const excelBuffer = XLSX.write(workbook, { bookType: 'xlsx', type: 'array' });

  // 使用 file-saver 保存文件
  const data = new Blob([excelBuffer], { type: 'application/octet-stream' }); // MIME类型
  saveAs(data, `network_metrics_result_${new Date().toISOString()}.xlsx`); // 设置文件名

  console.log('Excel文件下载已触发');
}


// 移除 onMounted 时的自动启动，让用户手动点击开始
// onMounted(() => {
//   console.log('网络监控组件已挂载');
// })

onUnmounted(() => {
  // 组件卸载时停止监控，清理定时器
  stopMonitoring()
})

// 监听 targetUrl 变化，清空错误信息
watch(targetUrl, (newValue, oldValue) => {
  if (newValue && urlError.value) {
    urlError.value = ''; // 用户开始输入时清空错误信息
  }
  // 如果监控中URL改变，可能需要提示用户或停止当前监控
  if(isMonitoring.value && newValue !== oldValue) { // 确保是值真的改变了
    console.warn('监控中改变了目标URL，请先停止监控再重新开始。');
    // 可以在这里自动停止监控
    // stopMonitoring();
  }
});

// 监听 metrics 长度，当数据变化时（包括新增和清空）处理分页和排序
watch(metrics, (newValue) => {
  // 当 metrics 数据变化时，保持在当前页，除非当前页变空了
  // 或者在清空数据时重置分页 (已在 resetChartAndMetrics 中处理)

  // 图表数据也与 metrics 同步长度
  if (newValue.length === 0 && chartData.value.labels.length > 0) {
    chartData.value = { // 清空图表数据
      labels: [],
      datasets: chartData.value.datasets.map(ds => ({ ...ds, data: [] }))
    };
  }

  // 当 metrics 变化时，如果当前页码超出了总页数，则回退到最后一页或第一页
  const totalPages = Math.ceil(newValue.length / pageSize.value);
  if (currentPage.value > totalPages && totalPages > 0) {
    currentPage.value = totalPages; // 回到最后一页
  } else if (totalPages === 0) {
    currentPage.value = 1; // 没有数据时回到第一页
  }

}, { deep: true });

// **新增：计算属性，用于获取当前页应该显示的数据（已排序和分页）**
const paginatedAndReversedMetrics = computed(() => {
  // 1. 复制原始数据，避免修改 ref
  const data = [...metrics.value];

  // 2. 按时间戳降序排序 (最新的在前面)
  // 注意：这里的 timestamp 是 u64 类型，直接相减可能溢出或不精确，
  // 但用于比较大小通常没问题，或者转换为 Number 或 BigInt 再比较
  data.sort((a, b) => b.metrics.timestamp - a.metrics.timestamp); // b - a 实现降序

  // 3. 计算当前页数据的起始和结束索引
  const startIndex = (currentPage.value - 1) * pageSize.value;
  const endIndex = startIndex + pageSize.value;

  // 4. 截取当前页的数据
  return data.slice(startIndex, endIndex);
});


// **新增：处理页码变化的函数**
const handlePageChange = (newPage: number) => {
  console.log(`切换到页面: ${newPage}`);
  currentPage.value = newPage;
  // computed 属性会自动更新表格数据
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
      <Line :data="chartData" :options="chartOptions" ref="chartRef" v-if="metrics.length > 0" /> <div v-else class="no-data-message">暂无图表数据</div>
    </div>

    <div class="chart-actions" v-if="metrics.length > 0">
      <el-button @click="downloadChartImage">下载图表图片 (JPG)</el-button>
    </div>


    <div class="metrics-table">
      <h3>最近检测结果 (总计 {{ metrics.length }} 条)</h3> <el-table :data="paginatedAndReversedMetrics" style="width: 100%">
      <el-table-column prop="metrics.timestamp" label="时间" width="180">
        <template #default="scope">
          {{ new Date(scope.row.metrics.timestamp).toLocaleString() }}
        </template>
      </el-table-column>
      <el-table-column prop="metrics.latency" label="延迟(ms)" width="120" />
      <el-table-column prop="metrics.packet_loss" label="丢包率(%)" width="120" />
<!--      <el-table-column prop="metrics.bandwidth" label="带宽" width="120" />-->
      <el-table-column prop="anomaly_score" label="异常分数" width="120" />
      <el-table-column prop="metrics.is_anomaly" label="是否异常" width="120">
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

    </div>

    <div class="table-actions" v-if="metrics.length > 0">
      <el-button @click="downloadMetricsAsExcel" type="success" :disabled="metrics.length === 0">下载全部结果为Excel (.xlsx)</el-button>
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

/* URL错误信息样式 */
.url-error-message {
  color: #f56c6c; /* Element UI Danger Red */
  margin-bottom: 15px;
  padding: 10px;
  background-color: #fef0f0; /* Light Red Background */
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
  display: flex; /* 使用flex布局居中无数据消息 */
  justify-content: center;
  align-items: center;
}

/* 新增：无数据消息样式 */
.no-data-message {
  color: #909399; /* Element UI Info Color */
  font-size: 1.2em;
}

/* 新增：图表和表格操作按钮容器样式 */
.chart-actions,
.table-actions {
  margin-top: 10px;
  margin-bottom: 20px;
  text-align: right; /* 按钮靠右对齐 */
}


.metrics-table {
  margin-top: 20px;
}

/* 新增：分页组件容器样式 */
.el-pagination {
  justify-content: flex-end; /* 让分页组件内容靠右对齐 */
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