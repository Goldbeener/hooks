<script setup>
import { ref, computed, onMounted } from "vue";
import { load } from "@tauri-apps/plugin-store";

const emit = defineEmits(["back"]);

const topics = ref([]);
const copyToast = ref(false);
const viewMode = ref("project"); // "project" | "day"

onMounted(async () => {
  const store = await load("topics.json", { autoSave: false });
  const saved = await store.get("topics");
  if (saved) topics.value = saved;
});

const weekStart = new Date();
const _day = weekStart.getDay();
const _diff = _day === 0 ? 6 : _day - 1; // 距本周一的天数
weekStart.setDate(weekStart.getDate() - _diff);
weekStart.setHours(0, 0, 0, 0);
const weekStartTs = weekStart.getTime();

function isWeekDone(item) {
  return item.done && item.doneAt >= weekStartTs;
}

function isWeekProgress(item) {
  if (item.done) return false;
  const t = item.progressAt;
  return !!(t && item.progress && t >= weekStartTs);
}

function dayLabel(ts) {
  const d = new Date(ts);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const itemDay = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  const diffDays = Math.round((today - itemDay) / 86400000);
  if (diffDays === 0) return "今天";
  if (diffDays === 1) return "昨天";
  if (diffDays === 2) return "前天";
  const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return weekdays[d.getDay()];
}

const reviewTopics = computed(() =>
  topics.value
    .filter((t) => t.items.some((i) => isWeekDone(i) || isWeekProgress(i)))
    .map((t) => ({
      ...t,
      doneItems: t.items.filter(isWeekDone).sort((a, b) => a.doneAt - b.doneAt),
      progressItems: t.items.filter(isWeekProgress).sort((a, b) => (b.progressAt || 0) - (a.progressAt || 0)),
    }))
);

const priorityLabel = { high: "高", medium: "中", low: "低" };

function formatTime(ts) {
  const d = new Date(ts);
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  return `${dayLabel(ts)} ${hh}:${mm}`;
}

function formatDate(ts) {
  const d = new Date(ts);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}.${m}.${day}`;
}

function naturalDays(fromTs, toTs) {
  const fromDay = new Date(new Date(fromTs).toDateString());
  const toDay = new Date(new Date(toTs).toDateString());
  return Math.round((toDay - fromDay) / 86400000);
}

function dayStartTs(ts) {
  const d = new Date(ts);
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

function buildJsonData() {
  const result = [];
  for (const topic of reviewTopics.value) {
    for (const item of topic.doneItems) {
      const days = naturalDays(item.id, item.doneAt);
      const name = `${topic.title} - ${item.text}`;
      const hours = Math.max(1, days) * 8;
      result.push({
        startDate: formatDate(item.id),
        lastUpdatedDate: formatDate(item.doneAt),
        hours,
        cardId: null,
        content: name,
        duration: [dayStartTs(item.id), dayStartTs(item.doneAt)],
        plannedHours: hours,
      });
    }
    for (const item of topic.progressItems) {
      const now = Date.now();
      const days = naturalDays(item.id, now);
      const name = `${topic.title} - ${item.text}（${item.progress}%）`;
      const hours = Math.max(1, days) * 8;
      result.push({
        name,
        startDate: formatDate(item.id),
        lastUpdatedDate: formatDate(item.progressAt),
        hours,
        cardId: null,
        content: name,
        duration: [dayStartTs(item.id), dayStartTs(item.progressAt)],
        plannedHours: hours,
      });
    }
  }
  return result;
}

async function copyToClipboard() {
  let content;
  if (viewMode.value === "day") {
    content = JSON.stringify(buildJsonData(), null, 2); // Task 3 will replace this
  } else {
    const lines = [];
    for (const topic of reviewTopics.value) {
      lines.push(`【${topic.title}】`);
      for (const item of topic.doneItems) {
        const days = naturalDays(item.id, item.doneAt);
        const dateRange = `${formatDate(item.id)} - ${formatDate(item.doneAt)}`;
        const dayStr = days === 0 ? `${dateRange}，当天完成` : `${dateRange}，历时${days}天`;
        lines.push(`  · ${item.text}【已完成】（${dayStr}）`);
      }
      for (const item of topic.progressItems) {
        const now = Date.now();
        const days = naturalDays(item.id, now);
        const dateRange = `${formatDate(item.id)} - ${formatDate(now)}`;
        const dayStr = days === 0 ? `${dateRange}，当天` : `${dateRange}，已历时${days}天`;
        lines.push(`  · ${item.text}【进度${item.progress}%】（${dayStr}）`);
      }
    }
    content = lines.join("\n");
  }
  await navigator.clipboard.writeText(content);
  copyToast.value = true;
  setTimeout(() => (copyToast.value = false), 2000);
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <button class="back-btn" @click="emit('back')" title="返回">‹</button>
      <span class="page-title">本周完成</span>
      <div class="seg-control">
        <button class="seg-btn" :class="{ active: viewMode === 'project' }" @click="viewMode = 'project'">项目</button>
        <button class="seg-btn" :class="{ active: viewMode === 'day' }" @click="viewMode = 'day'">按天</button>
      </div>
      <button class="copy-btn" @click="copyToClipboard" title="复制到剪贴板">复制</button>
    </div>
    <div class="format-hint">
      <span class="hint-dot"></span>
      {{ viewMode === 'project' ? '复制为文本格式' : '复制为 JSON 格式，拖拽调整各任务工时' }}
    </div>

    <transition name="toast">
      <div v-if="copyToast" class="toast">已复制到剪贴板</div>
    </transition>

    <div v-if="reviewTopics.length === 0" class="empty">本周还没有完成的子任务</div>

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
  flex-shrink: 0;
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

.copy-btn {
  background: #1a1a1a;
  color: white;
  border: none;
  border-radius: 10px;
  padding: 6px 14px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}

.seg-control {
  margin-left: auto;
  display: flex;
  background: rgba(0, 0, 0, 0.06);
  border-radius: 9px;
  padding: 2px;
  gap: 2px;
}

.seg-btn {
  border: none;
  border-radius: 7px;
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  color: #888;
  background: transparent;
}

.seg-btn.active {
  background: white;
  color: #1a1a1a;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.format-hint {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 0 6px;
  font-size: 11px;
  color: #bbb;
}

.hint-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: #ccc;
  flex-shrink: 0;
}

.copy-btn:active {
  opacity: 0.7;
}

.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.75);
  color: white;
  font-size: 13px;
  padding: 8px 18px;
  border-radius: 20px;
  pointer-events: none;
  z-index: 100;
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.25s;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
}
</style>
