<script setup>
import { ref, computed, onMounted } from "vue";
import { load } from "@tauri-apps/plugin-store";

const emit = defineEmits(["back"]);

const topics = ref([]);
const showYesterday = ref(false);

onMounted(async () => {
  const store = await load("topics.json", { autoSave: false });
  const saved = await store.get("topics");
  if (saved) topics.value = saved;
});

const todayStart = new Date();
todayStart.setHours(0, 0, 0, 0);
const todayStartTs = todayStart.getTime();
const yesterdayStartTs = todayStartTs - 86400000;

function isDayDone(item) {
  if (showYesterday.value) {
    return item.done && item.doneAt >= yesterdayStartTs && item.doneAt < todayStartTs;
  }
  return item.done && item.doneAt >= todayStartTs;
}

function isDayProgress(item) {
  if (item.done) return false; // 已完成的走 isDayDone 逻辑
  const t = item.progressAt;
  if (!t || !item.progress) return false;
  if (showYesterday.value) {
    return t >= yesterdayStartTs && t < todayStartTs;
  }
  return t >= todayStartTs;
}

const reviewTopics = computed(() =>
  topics.value
    .filter((t) => t.items.some((i) => isDayDone(i) || isDayProgress(i)))
    .map((t) => ({
      ...t,
      doneItems: t.items.filter(isDayDone).sort((a, b) => a.doneAt - b.doneAt),
      progressItems: t.items.filter(isDayProgress).sort((a, b) => (b.progressAt || 0) - (a.progressAt || 0)),
    }))
);

const priorityLabel = { high: "高", medium: "中", low: "低" };

function formatTime(ts) {
  const d = new Date(ts);
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  return `${hh}:${mm}`;
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <button class="back-btn" @click="emit('back')" title="返回">‹</button>
      <div class="tab-group">
        <button class="tab-btn" :class="{ active: !showYesterday }" @click="showYesterday = false">今日</button>
        <button class="tab-btn" :class="{ active: showYesterday }" @click="showYesterday = true">昨日</button>
      </div>
    </div>

    <div v-if="reviewTopics.length === 0" class="empty">
      {{ showYesterday ? "昨天没有完成的子任务" : "今天还没有完成的子任务" }}
    </div>

    <div v-else class="list">
      <div v-for="topic in reviewTopics" :key="topic.id" class="topic-card" :class="topic.priority">
        <div class="topic-header">
          <span class="topic-title">{{ topic.title }}</span>
          <span class="priority-badge" :class="topic.priority">{{ priorityLabel[topic.priority] }}</span>
        </div>
        <div class="items">
          <div v-for="item in topic.doneItems" :key="item.id" class="item item-done">
            <span class="checkbox done-check">✓</span>
            <span class="item-text">{{ item.text }}</span>
            <span class="item-time">{{ formatTime(item.doneAt) }}</span>
          </div>
          <div v-for="item in topic.progressItems" :key="item.id" class="item item-progress">
            <span class="checkbox progress-check">…</span>
            <span class="item-text">{{ item.text }}</span>
            <span class="progress-badge">{{ item.progress }}%</span>
            <span class="item-time">{{ formatTime(item.progressAt) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 20px 16px;
  gap: 12px;
  background: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(12px);
  border-radius: 16px;
}

.page-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 4px;
}

.back-btn {
  background: white;
  border: none;
  border-radius: 10px;
  width: 36px;
  height: 36px;
  font-size: 22px;
  cursor: pointer;
  color: #666;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  padding-bottom: 2px;
  flex-shrink: 0;
}

.tab-group {
  display: flex;
  background: rgba(0, 0, 0, 0.06);
  border-radius: 10px;
  padding: 3px;
  gap: 2px;
}

.tab-btn {
  border: none;
  border-radius: 8px;
  padding: 5px 14px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  color: #888;
  background: transparent;
  transition: background 0.15s, color 0.15s;
}

.tab-btn.active {
  background: white;
  color: #1a1a1a;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  color: #bbb;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

.list::-webkit-scrollbar {
  display: none;
}

.list::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
  border-radius: 4px;
}

.topic-card {
  border-radius: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}

.topic-card.high {
  background: #fff5f5;
}

.topic-card.medium {
  background: #fffdf0;
}

.topic-card.low {
  background: #f6fff8;
}

.topic-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 13px 14px 10px;
}

.topic-title {
  flex: 1;
  font-size: 15px;
  font-weight: 600;
  color: #1a1a1a;
}

.priority-badge {
  border-radius: 8px;
  padding: 3px 8px;
  font-size: 12px;
  font-weight: 600;
}

.priority-badge.high {
  background: #ffe0e0;
  color: #e53e3e;
}

.priority-badge.medium {
  background: #fef3c7;
  color: #d97706;
}

.priority-badge.low {
  background: #dcfce7;
  color: #38a169;
}

.items {
  padding: 0 14px 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  background: rgba(255, 255, 255, 0.7);
  border-radius: 10px;
}

.checkbox {
  font-size: 15px;
  width: 18px;
  flex-shrink: 0;
}

.done-check {
  color: #38a169;
}

.progress-check {
  color: #d97706;
}

.item-text {
  flex: 1;
  font-size: 14px;
  color: #555;
}

.item-done .item-text {
  text-decoration: line-through;
}

.progress-badge {
  font-size: 11px;
  font-weight: 700;
  color: #d97706;
  background: #fef3c7;
  border-radius: 6px;
  padding: 1px 6px;
  white-space: nowrap;
}

.item-time {
  font-size: 11px;
  color: #bbb;
  white-space: nowrap;
}
</style>
