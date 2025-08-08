<template>
  <div>
    <h1>Updaters Status</h1>
    <ul>
      <li v-for="upt in updaters" :key="upt.updater">
        {{ upt.updater }}: {{ upt.status }}
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const updaters = ref([
  { updater: 'VRChat', status: 'Unknown' },
  { updater: 'Discord', status: 'Unknown' },
]);

onMounted(async () => {
  try {
    updaters.value = await invoke('get_updaters_state');
    console.log("get_updaters_state: ", updaters.value);
  } catch (e) {
    console.error(e);
  }
});
</script>
