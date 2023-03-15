<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog"
import { listen } from "@tauri-apps/api/event";

const selectedDir = ref("Choose Directory");
const working = ref(false);

async function processDirectory() {
  if (selectedDir.value == "" || selectedDir.value == "Choose Directory") {
    alert("please select a valid directory")
    return;
  };
  working.value = true;
  await invoke("process_directory", { directory: selectedDir.value, customJava: javaExe.value != "Choose Executable" ? javaExe.value : "" });
  working.value = false;
}

async function pickDirectory() {
  const newDir = await open({ directory: true });
  if (newDir == null) return;
  selectedDir.value = newDir;
}

const latestLog = ref("No Last Tast");

onMounted(async () => {
  await listen("analysis_info", (event) => {
    latestLog.value = event.payload;
  });
});

const advanced = ref(false);

const javaExe = ref("Choose Executable");

async function pickJava() {
  const newDir = await open({
    filters: [{
      name: "Executable",
      extensions: ["exe"]
    }]
  });
  if (newDir == null) return;
  javaExe.value = newDir;
}

</script>

<template>
  <div class="bg-gray-700 w-[50vh] max-w-full h-[40vh] flex flex-col place-items-center justify-center gap-4 rounded relative">
    <button @click="advanced = !advanced" class="absolute top-3 right-5 hover:animate-spin"><img src="../assets/gear.svg" class="h-[5vh]" /></button>
    <div class="flex flex-row place-items-center justify-between h-[5vh] gap-1">
      <h1 class=" text-xl">UmlBot</h1>
      <img src="../assets/robot-icon.svg" class=" h-full" />
    </div>
    <button @click="pickDirectory" class="p-3 truncate overflow-hidden bg-gray-500 w-5/6 h-[10vh] rounded">{{ selectedDir }}</button>
    <button v-if="!working" @click="processDirectory" class="p-3 bg-green-500 hover:bg-green-700 w-5/6 h-[10vh] rounded flex flex-row place-items-center justify-between">
      <p>Process</p>
      <img src="../assets/arrow-circle-right.svg" class=" h-full" />
    </button>
    <div v-else class="p-3 bg-red-700 w-5/6 h-[10vh] rounded flex flex-row place-items-center justify-around">
      <p>Working</p>
    </div>
    <p class="px-3 bg-gray-500 w-5/6 h-[3vh] rounded text-xs">{{ latestLog }}</p>
  </div>
  <div v-if="advanced" class="bg-gray-700 w-[50vh] max-w-full h-[12vh] flex flex-col place-items-center justify-center gap-4 rounded relative">
    <div class="w-5/6">
      <label for="customJava" class="w-full text-left text-xs">Custom Java Directory (containing java.exe)</label>
      <div id="customJava" class="bg-gray-500 w-full h-[7vh] rounded flex flex-row justify-between place-items-center p-2">
        <button @click="pickJava" class="truncate overflow-hidden">{{ javaExe }}</button>
        <button @click="javaExe = 'Choose Directory'" class="h-[5vh] w-[8vh] rounded bg-gray-600"><img src="../assets/reset.svg" class="h-full" /></button>
      </div>
    </div>
  </div>
</template>
