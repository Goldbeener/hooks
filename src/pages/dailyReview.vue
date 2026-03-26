<script setup>
import { ref, computed, onMounted } from "vue";
import { load } from "@tauri-apps/plugin-store";

const emit = defineEmits(["back"]);

const topics = ref([]);

onMounted(async () => {
  const store = await load("topics.json", { autoSave: false });
  const saved = await store.get("topics");
  if (saved) topics.value = saved;
});

const todayStart = new Date();
todayStart.setHours(0, 0, 0, 0);
const todayStartTs = todayStart.getTime();

function isTodayDone(item) {
  return item.done && item.doneAt >= todayStartTs;
}

const reviewTopics = computed(() =>
  topics.value
    .filter((t) => {
      const hasUnfinished = t.items.some((i) => !i.done);
      const hasTodayDone = t.items.some(isTodayDone);
      return hasUnfinished && hasTodayDone;
    })
    .map((t) => ({
      ...t,
      doneItems: t.items.filter(isTodayDone).sort((a, b) => a.doneAt - b.doneAt),
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
      <span class="page-title">今日完成</span>
    </div>

    <div v-if="reviewTopics.length === 0" class="empty">今天还没有完成的子任务</div>

    <div v-else class="list">
      <div v-for="topic in reviewTopics" :key="topic.id" class="topic-card" :class="topic.priority">
        <div class="topic-header">
          <span class="topic-title">{{ topic.title }}</span>
          <span class="priority-badge" :class="topic.priority">{{ priorityLabel[topic.priority] }}</span>
        </div>
        <div class="items">
          <div v-for="item in topic.doneItems" :key="item.id" class="item">
            <span class="checkbox">✓</span>
            <span class="item-text">{{ item.text }}</span>
            <span class="item-time">{{ formatTime(item.doneAt) }}</span>
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
}

.page-title {
  font-size: 17px;
  font-weight: 600;
  color: #1a1a1a;
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
  width: 4px;
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

.topic-card.high { background: #fff5f5; }
.topic-card.medium { background: #fffdf0; }
.topic-card.low { background: #f6fff8; }

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

.priority-badge.high { background: #ffe0e0; color: #e53e3e; }
.priority-badge.medium { background: #fef3c7; color: #d97706; }
.priority-badge.low { background: #dcfce7; color: #38a169; }

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
  color: #38a169;
  width: 18px;
  flex-shrink: 0;
}

.item-text {
  flex: 1;
  font-size: 14px;
  color: #555;
  text-decoration: line-through;
}

.item-time {
  font-size: 11px;
  color: #bbb;
  white-space: nowrap;
}
</style>
