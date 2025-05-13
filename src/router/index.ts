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
      // 自动重定向到 /yuketang 页面
      redirect: '/paiad-blog',
      // 子路由配置
      children: [
        {
          // 子路由路径（完整路径为 /yuketang）
          path: 'yu-ke-tang',
          // 路由名称
          name: 'Dashboard',
          // 懒加载 yuketang 页面组件
          component: () => import('../views/yuketang/index.vue'),
          // 路由元信息（可以用于显示标题、图标等）
          meta: { title: '雨课堂', icon: 'Odometer' }
        },
        {
          path: 'paiad-blog',
          name: 'Paiad Blog',
          component: () => import('../views/paiad-blog/index.vue'),
          meta: { title: '博客', icon: '' }
        },
        {
          path: 'more',
          name: '更多',
          component: () => import('../views/more/index.vue'),
          meta: { title: '更多', icon: 'Plus' }
        },
        {
          path: 'stress-test',
          name: '压测',
          component: () => import('../views/stress-test/index.vue'),
          meta: { title: '压测', icon: '' }
        },
        {
          path: 'file-exchange',
          name: '文件互传',
          component: () => import('../views/file-exchange/index.vue'),
          meta: { title: '文件互传', icon: '' }
        }
      ]
    }
  ]
})

// 导出路由实例，用于挂载到 Vue 应用中
export default router
