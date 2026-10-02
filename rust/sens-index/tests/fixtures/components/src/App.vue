<template>
  <button class="primary" @click="onClick">{{ formatDate(today) }}</button>
</template>

<script setup lang="ts">
import { formatDate } from "./dates";

const today = "2026-03-01";

function onClick(): void {
  console.log(today);
}

function unusedHelper(): number {
  return 1;
}
</script>

<style scoped>
.primary { color: black }
</style>
