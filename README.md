# 🚀 快速开始

## 🧪 开发模式

```bash
pnpm tauri dev
```

## 📦 打包发布

```bash
pnpm tauri build
```

---

# ⚙️ Tauri + Vue 3 + TypeScript 项目模板

这是一个基于 **Tauri + Vue 3 + TypeScript + Vite** 的桌面应用开发模板，默认采用 Vue 3 的 `<script setup>` 单文件组件语法，适合快速构建高性能跨平台桌面应用。

📘 想了解 `<script setup>` 的更多信息？请查阅官方文档：[Vue 3 script setup 使用指南](https://cn.vuejs.org/api/sfc-script-setup.html)

---

## 💡 推荐开发工具

为了获得最佳开发体验，推荐使用以下插件和 IDE：

* [Visual Studio Code](https://code.visualstudio.com/)
* 插件推荐：

    * [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) —— Vue 3 专属语法支持
    * [Tauri VSCode 插件](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) —— 快速运行和打包 Tauri 应用
    * [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) —— Rust 开发增强工具

---

## 🧩 TypeScript 支持 `.vue` 文件导入说明

由于 TypeScript 默认无法识别 `.vue` 文件的组件类型，因此系统会默认将其视为普通组件类型处理。如果你希望在 `.vue` 导入中获取更完整的 props 类型提示（例如在手动使用 `h(...)` 调用时），可以启用 **Volar 的接管模式（Take Over Mode）**：

### ⚙️ 开启方式：

1. 在 VS Code 命令面板中运行 `Extensions: Show Built-in Extensions`；
2. 搜索并禁用 `TypeScript and JavaScript Language Features`；
3. 然后运行 `Developer: Reload Window` 重新加载窗口；
4. Volar 将自动启用接管模式。

🔗 了解更多：[Volar Take Over 模式介绍](https://github.com/johnsoncodehk/volar/discussions/471)

---

## 📌 项目特点

* 💻 使用 Tauri 构建桌面应用，体积小、性能高、跨平台
* 🛠️ Vue 3 + Vite，现代前端开发体验
* 🧠 TypeScript 强类型支持，减少运行时错误
* 🔩 支持 Rust 后端逻辑扩展

