<template>
  <div class="app-wrapper">
    <!-- 侧边栏 -->
    <div class="sidebar-container" :class="{ 'is-collapse': isCollapse }">
      <div class="logo-container">
        <img src="../assets/logo.png" alt="Logo" class="logo" v-if="!isCollapse">
        <img src="../assets/logo.png" alt="Logo" class="logo-small" v-else>
      </div>
      <el-menu
        :default-active="activeMenu"
        class="sidebar-menu"
        :collapse="isCollapse"
        background-color="#304156"
        text-color="#747474"
        active-text-color="#409EFF"
        router
      >
        <el-menu-item index="/paiad-blog">
          <el-icon><Icon icon="tdesign:broccoli" /></el-icon>
          <template #title>博客</template>
        </el-menu-item>
        <el-menu-item index="/yu-ke-tang">
          <el-icon><Icon icon="hugeicons:cloud"/></el-icon>
          <template #title>雨课堂</template>
        </el-menu-item>
        <el-menu-item index="/more">
          <el-icon><Plus /></el-icon>
          <template #title>更多</template>
        </el-menu-item>
      </el-menu>
    </div>

    <!-- 主要内容区域 -->
    <div class="main-container">
      <!-- 顶部导航栏 -->
      <div class="navbar">
        <div class="left">
          <el-icon class="fold-btn" @click="toggleSidebar">
            <Fold v-if="!isCollapse" />
            <Expand v-else />
          </el-icon>
        </div>
        <div class="right">
          <el-dropdown>
            <span class="user-info">
              <el-avatar :size="27" src="https://paiad.online/sunflower.png" style="background-color: #e9f5eb"/>
              <span class="username">管理员</span>
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item>个人信息</el-dropdown-item>
                <el-dropdown-item>退出登录</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>

      <!-- 内容区域 -->
      <div class="app-main">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon } from '@iconify/vue';
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import {Plus} from "@element-plus/icons-vue";

const route = useRoute()
const isCollapse = ref(true)
const activeMenu = computed(() => route.path)

const toggleSidebar = () => {
  isCollapse.value = !isCollapse.value
}
</script>

<style scoped>
.app-wrapper {
  display: flex;
  height: 100vh;
  width: 100%;
}

.sidebar-container {
  width: 210px;
  height: 100%;
  background-color: #ffffff;
  transition: width 0.3s;
  overflow: hidden;
}

.sidebar-container.is-collapse {
  width: 64px;
}

.logo-container {
  height: 60px;
  padding: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #ffffff;
}

.logo {
  height: 32px;
}

.logo-small {
  height: 32px;
}

.sidebar-menu {
  border-right: none;
}

.main-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.navbar {
  height: 60px;
  background-color: #fff;
  box-shadow: 0 1px 4px rgba(0, 21, 41, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
}

.fold-btn {
  font-size: 20px;
  cursor: pointer;
  color: #606266;
}

.user-info {
  display: flex;
  align-items: center;
  cursor: pointer;
}

.username {
  margin-left: 8px;
  color: #606266;
}

.app-main {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  background-color: #f0f2f5;
}

/* 移除黑色框线 */
.user-info:focus {
  outline: none;
}
</style>