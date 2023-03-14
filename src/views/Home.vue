<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog"

const selectedDir = ref("Choose Directory");

async function processDirectory() {
  if (selectedDir.value == "" || selectedDir.value == "Choose Directory") {
    alert("please select a valid directory")
    return;
  };
  await invoke("process_directory", { directory: selectedDir.value });
  alert("processing");
}

async function pickDirectory() {
  const newDir = await open({ directory: true });
  if (newDir == null) return;
  selectedDir.value = newDir;
}
</script>

<template>
  <div class=" bg-gray-700 w-[50vh] max-w-full h-[40vh] flex flex-col place-items-center justify-center gap-4 rounded">
    <h1 class=" text-xl">UmlBot 🤖</h1>
    <button @click="pickDirectory" class="p-3 truncate overflow-hidden bg-gray-500 w-5/6 h-[10vh] rounded">{{ selectedDir }}</button>
    <button @click="processDirectory" class="p-3 bg-green-500 hover:bg-green-700 w-5/6 h-[10vh] rounded flex flex-row place-items-center justify-between">
      Process
      <img src="../assets/arrow-circle-right.svg" class=" h-full" />
    </button>
  </div>
</template>
