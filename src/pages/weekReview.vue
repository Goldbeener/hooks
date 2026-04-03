<script setup>
import { ref, computed, onMounted } from "vue";
import { load } from "@tauri-apps/plugin-store";

const emit = defineEmits(["back"]);

const topics = ref([]);
const copyToast = ref(false);
const copyFormat = ref("text"); // "text" | "json"

onMounted(async () => {
  const store = await load("topics.json", { autoSave: false });
  const saved = await store.get("topics");
  if (saved) topics.value = saved;
});

const weekStart = new Date();
weekStart.setDate(weekStart.getDate() - 6);
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

function buildJsonData() {
  const result = [];
  for (const topic of reviewTopics.value) {
    for (const item of topic.doneItems) {
      const days = naturalDays(item.id, item.doneAt);
      result.push({
        name: `${topic.title} - ${item.text}`,
        startDate: formatDate(item.id),
        lastUpdatedDate: formatDate(item.doneAt),
        hours: Math.max(1, days) * 8,
      });
    }
    for (const item of topic.progressItems) {
      const now = Date.now();
      const days = naturalDays(item.id, now);
      result.push({
        name: `${topic.title} - ${item.text}（${item.progress}%）`,
        startDate: formatDate(item.id),
        lastUpdatedDate: formatDate(item.progressAt),
        hours: Math.max(1, days) * 8,
      });
    }
  }
  return result;
}

async function copyToClipboard() {
  let content;
  if (copyFormat.value === "json") {
    content = JSON.stringify(buildJsonData(), null, 2);
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
      <template v-if="reviewTopics.length > 0">
        <label class="format-switch">
          <span :class="{ active: copyFormat === 'text' }">文本</span>
          <span class="switch-track" @click="copyFormat = copyFormat === 'text' ? 'json' : 'text'">
            <span class="switch-thumb" :class="{ json: copyFormat === 'json' }"></span>
          </span>
          <span :class="{ active: copyFormat === 'json' }">JSON</span>
        </label>
        <button class="copy-btn" @click="copyToClipboard" title="复制到剪贴板">复制</button>
      </template>
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
  margin-left: auto;
  background: #1a1a1a;
  color: white;
  border: none;
  border-radius: 10px;
  padding: 6px 14px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}

.format-switch {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: #aaa;
  cursor: default;
  user-select: none;
}

.format-switch span.active {
  color: #1a1a1a;
  font-weight: 600;
}

.switch-track {
  width: 30px;
  height: 17px;
  background: #ddd;
  border-radius: 9px;
  position: relative;
  cursor: pointer;
  transition: background 0.2s;
  flex-shrink: 0;
}

.switch-track:has(.switch-thumb.json) {
  background: #1a1a1a;
}

.switch-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 13px;
  height: 13px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.switch-thumb.json {
  transform: translateX(13px);
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
