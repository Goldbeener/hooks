<script setup>
import { ref, computed, watch, onMounted } from "vue";
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
const MS_PER_DAY = 86400000;

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

function getWeekdayLabel(dayTs) {
  const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return weekdays[new Date(dayTs).getDay()];
}

function getDateLabel(dayTs) {
  const d = new Date(dayTs);
  return `${d.getMonth() + 1}月${d.getDate()}日`;
}

const dayGroups = computed(() => {
  const entryMap = new Map(); // key: dayTs, value: DayEntry[]

  for (const topic of topics.value) {
    for (const item of topic.items) {
      // 已完成条目
      if (item.done && item.doneAt >= weekStartTs) {
        const dayTs = dayStartTs(item.doneAt);
        const entry = {
          key: `${topic.id}-${item.id}-done`,
          dayTs,
          topicTitle: topic.title,
          itemId: item.id,
          itemText: item.text,
          progress: null,
          isDone: true,
          sortTs: item.doneAt,
        };
        if (!entryMap.has(dayTs)) entryMap.set(dayTs, []);
        entryMap.get(dayTs).push(entry);

        // 跨天：若 progressAt 存在且不在同一天，额外生成进度中条目
        // 仅当 progressAt 也在本周内时，才生成跨天进度条目
        if (item.progressAt && dayStartTs(item.progressAt) !== dayTs && item.progressAt >= weekStartTs) {
          const progDayTs = dayStartTs(item.progressAt);
          const progEntry = {
            key: `${topic.id}-${item.id}-progress`,
            dayTs: progDayTs,
            topicTitle: topic.title,
            itemId: item.id,
            itemText: item.text,
            progress: item.progress,
            isDone: false,
            sortTs: item.progressAt,
          };
          if (!entryMap.has(progDayTs)) entryMap.set(progDayTs, []);
          entryMap.get(progDayTs).push(progEntry);
        }
      }

      // 进度中条目（未完成）
      if (!item.done && item.progress > 0 && item.progressAt && item.progressAt >= weekStartTs) {
        const dayTs = dayStartTs(item.progressAt);
        const entry = {
          key: `${topic.id}-${item.id}-progress`,
          dayTs,
          topicTitle: topic.title,
          itemId: item.id,
          itemText: item.text,
          progress: item.progress,
          isDone: false,
          sortTs: item.progressAt,
        };
        if (!entryMap.has(dayTs)) entryMap.set(dayTs, []);
        entryMap.get(dayTs).push(entry);
      }
    }
  }

  // 排序并组装 DayGroup
  const sortedDays = [...entryMap.keys()].sort((a, b) => a - b);
  return sortedDays.map((dayTs) => {
    const entries = entryMap.get(dayTs).sort((a, b) => a.sortTs - b.sortTs);
    return {
      dayTs,
      dayLabel: getWeekdayLabel(dayTs),
      dateLabel: getDateLabel(dayTs),
      entries,
    };
  });
});

const dayHours = ref(new Map()); // key: entry.key, value: hours

// When dayGroups changes, initialize default hours (preserve user-dragged values)
watch(
  dayGroups,
  (groups) => {
    const newMap = new Map();
    for (const group of groups) {
      const count = group.entries.length;
      const defaultHours = Math.max(0.5, Math.floor((8 / count) * 2) / 2);
      for (const entry of group.entries) {
        // Preserve values the user has already dragged
        newMap.set(entry.key, dayHours.value.get(entry.key) ?? defaultHours);
      }
    }
    dayHours.value = newMap;
  },
  { immediate: true }
);

function getDayTotal(group) {
  return group.entries.reduce((sum, e) => sum + (dayHours.value.get(e.key) ?? 0), 0);
}

function startHoursDrag(event, entryKey) {
  event.stopPropagation();
  event.preventDefault();
  const barEl = event.currentTarget;

  function onMove(e) {
    const rect = barEl.getBoundingClientRect();
    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    // Map ratio to 0.5–24h in 0.5h steps
    const raw = ratio * 24;
    const stepped = Math.max(0.5, Math.round(raw * 2) / 2);
    dayHours.value = new Map(dayHours.value).set(entryKey, stepped);
  }

  function onUp() {
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", onUp);
    window.removeEventListener("touchmove", onMove);
    window.removeEventListener("touchend", onUp);
    window.removeEventListener("touchcancel", onUp);
  }

  onMove(event);
  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", onUp);
  window.addEventListener("touchmove", onMove, { passive: false });
  window.addEventListener("touchend", onUp);
  window.addEventListener("touchcancel", onUp);
}

function buildDayJsonData() {
  const result = [];
  for (const group of dayGroups.value) {
    const dayEnd = group.dayTs + MS_PER_DAY - 1; // 当天 23:59:59.999
    for (const entry of group.entries) {
      const hours = dayHours.value.get(entry.key) ?? 0;
      const content = entry.isDone
        ? `${entry.topicTitle} - ${entry.itemText}`
        : `${entry.topicTitle} - ${entry.itemText}（${entry.progress}%）`;
      result.push({
        startDate: formatDate(entry.itemId),
        lastUpdatedDate: formatDate(group.dayTs),
        hours,
        cardId: null,
        content,
        duration: [group.dayTs, dayEnd],
        plannedHours: hours,
      });
    }
  }
  return result;
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
    content = JSON.stringify(buildDayJsonData(), null, 2);
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
