// 从 vue-router 中导入创建路由的方法和使用 HTML5 history 模式的方法
import { createRouter, createWebHistory } from 'vue-router'
// 导入主布局组件，用于作为页面的基础框架
import Layout from '../layout/index.vue'

// 创建路由实例
const router = createRouter({
  // 使用 history 模式（地址栏中没有 #，更接近正常网址）
  history: createWebHistory(),
  // 定义路由表
  routes: [
    {
      // 根路径
      path: '/',
      // 使用的组件为主布局 Layout
      component: Layout,
      // 自动重定向到 /dashboard 页面
      redirect: '/dashboard',
      // 子路由配置
      children: [
        {
          // 子路由路径（完整路径为 /dashboard）
          path: 'dashboard',
          // 路由名称
          name: 'Dashboard',
          // 懒加载 dashboard 页面组件
          component: () => import('../views/dashboard/index.vue'),
          // 路由元信息（可以用于显示标题、图标等）
          meta: { title: '雨课堂', icon: 'Odometer' }
        },
        {
          path: 'yu-ke-tang',
          name: 'Yu-Ke-Tang',
          component: () => import('../views/tools/yu-ke-tang.vue'),
          meta: { title: '更多', icon: 'Plus' }
        }
      ]
    }
  ]
})

// 导出路由实例，用于挂载到 Vue 应用中
export default router
