<template>
  <div class="container">
    <h1>👋 欢迎使用 Tauri 示例</h1>

    <div class="input-group">
      <label for="name">请输入你的名字：</label>
      <input id="name" v-model="name" placeholder="例如：Alice" />
    </div>

    <button @click="greet">👂 听听 Rust 怎么说</button>

    <div class="output" v-if="greetMsg">
      <h2>🎉 返回的信息：</h2>
      <p>{{ greetMsg }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<style scoped>
.container {
  max-width: 500px;
  margin: 50px auto;
  padding: 20px;
  font-family: "Arial", sans-serif;
  border: 1px solid #ccc;
  border-radius: 8px;
  background-color: #fafafa;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.input-group {
  margin-bottom: 20px;
}

input {
  padding: 8px;
  font-size: 16px;
  width: 100%;
  box-sizing: border-box;
  margin-top: 8px;
}

button {
  padding: 10px 20px;
  font-size: 16px;
  background-color: #0066ff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background-color: #0051cc;
}

.output {
  margin-top: 30px;
  padding: 15px;
  background-color: #e8709a;
  border: 1px solid rgba(170, 215, 191, 0.15);
  border-radius: 5px;
}
</style>
