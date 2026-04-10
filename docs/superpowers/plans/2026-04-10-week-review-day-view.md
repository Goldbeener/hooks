# 本周完成·按天视图 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 `weekReview.vue` 页面新增「按天视图」模式，将本周任务按自然日聚合展示，支持拖拽调整各任务当天工时，复制时输出 JSON。

**Architecture:** 所有改动集中在单文件 `src/pages/weekReview.vue`，分为四个部分：①移除旧的格式开关，替换为「项目/按天」分段控件 + 格式说明文字；②新增 `buildDayEntries` 聚合函数，将 topics 数据按天展开为条目列表；③新增 `dayHours` 响应式状态保存每个条目的临时工时及拖拽逻辑；④新增 `buildDayJsonData` 生成按天 JSON。所有工时数据纯内存保存，不写入 store。

**Tech Stack:** Vue 3 Composition API (`<script setup>`), `@tauri-apps/plugin-store`（只读），原生 DOM 拖拽事件（与现有 `startDragProgress` 同模式）

---

## File Structure

| 文件 | 变更类型 | 说明 |
|------|---------|------|
| `src/pages/weekReview.vue` | Modify | 全部改动在此一个文件内完成 |

---

## Task 1: 替换格式开关为「项目/按天」分段控件

**Files:**
- Modify: `src/pages/weekReview.vue`

### 目标
- 删除 `copyFormat` ref 及所有对它的引用
- 新增 `viewMode` ref：`"project" | "day"`，默认 `"project"`
- 页头：将旧的 `format-switch` + `copy-btn` 区域替换为分段控件 + 复制按钮
- 页头下方新增一行格式说明文字（跟随 `viewMode` 切换）

### Script 变更

- [ ] **Step 1: 删除旧状态，新增 viewMode**

在 `<script setup>` 中，将：
```js
const copyToast = ref(false);
const copyFormat = ref("text"); // "text" | "json"
```
替换为：
```js
const copyToast = ref(false);
const viewMode = ref("project"); // "project" | "day"
```

- [ ] **Step 2: 更新 copyToClipboard 函数**

将现有 `copyToClipboard` 中 `copyFormat.value === "json"` 的判断，改为 `viewMode.value === "day"`（按天视图复制 JSON，项目视图复制文本）。暂时仍调用现有的 `buildJsonData()`，后续 Task 3 再替换为 `buildDayJsonData()`：

```js
async function copyToClipboard() {
  let content;
  if (viewMode.value === "day") {
    content = JSON.stringify(buildJsonData(), null, 2); // Task 3 会替换
  } else {
    // 文本逻辑保持不变
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
```

- [ ] **Step 3: 更新 template — 页头区域**

将 `<template>` 中的 `<div class="page-header">` 内容替换为：

```html
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
```

注意：移除旧的 `<template v-if="reviewTopics.length > 0">` 包裹（复制按钮改为始终显示，无数据时点击复制空内容即可）。

- [ ] **Step 4: 删除旧 format-switch 相关 CSS，新增分段控件 CSS**

在 `<style scoped>` 中，删除以下 CSS 类：`.format-switch`、`.format-switch span.active`、`.switch-track`、`.switch-track:has(.switch-thumb.json)`、`.switch-thumb`、`.switch-thumb.json`。

新增：

```css
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
```

- [ ] **Step 5: 确认页面渲染正常**

运行 `pnpm tauri dev`，确认：
- 页头显示「项目 / 按天」分段控件
- 点击切换时格式提示文字随之变化
- 项目视图下，现有卡片列表正常显示
- 复制按钮在两种视图下均可点击

- [ ] **Step 6: Commit**

```bash
git add src/pages/weekReview.vue
git commit -m "feat(weekReview): replace format toggle with project/day segmented control"
```

---

## Task 2: 新增 buildDayEntries 聚合函数

**Files:**
- Modify: `src/pages/weekReview.vue`

### 目标

实现 `buildDayEntries()` 函数，将 `topics.value` 展开为按天排列的条目结构，供按天视图渲染使用。

### 数据结构

```ts
// 每条「天-任务」条目
type DayEntry = {
  key: string;          // 唯一标识，如 "topic.id-item.id-done" 或 "topic.id-item.id-progress"
  dayTs: number;        // 该天 00:00 时间戳（用于分组和排序）
  topicTitle: string;
  itemId: number;       // item.id（即 item 创建时间戳）
  itemText: string;
  progress: number | null;  // null = 已完成条目；数字 = 进度中条目的进度值
  isDone: boolean;
  sortTs: number;       // 用于天内排序（doneAt 或 progressAt）
}

// 按天分组后的结构
type DayGroup = {
  dayTs: number;        // 该天 00:00 时间戳
  dayLabel: string;     // "周一"、"周二"…
  dateLabel: string;    // "4月7日"
  entries: DayEntry[];
}
```

- [ ] **Step 1: 确认复用现有 `dayStartTs` 函数**

现有代码已有 `dayStartTs(ts)` 函数，功能与 `getDayStart` 完全相同，**无需新增**。后续所有步骤中涉及「取某天 00:00 时间戳」均调用 `dayStartTs(ts)`。

- [ ] **Step 2: 新增 `getDayLabel` 函数**

```js
function getDayLabel(dayTs) {
  const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return weekdays[new Date(dayTs).getDay()];
}

function getDateLabel(dayTs) {
  const d = new Date(dayTs);
  return `${d.getMonth() + 1}月${d.getDate()}日`;
}
```

- [ ] **Step 3: 新增 `buildDayEntries` computed**

```js
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
      dayLabel: getDayLabel(dayTs),
      dateLabel: getDateLabel(dayTs),
      entries,
    };
  });
});
```

- [ ] **Step 4: 验证 computed 输出（浏览器控制台）**

在 `pnpm tauri dev` 环境中，在 Vue DevTools 或控制台验证 `dayGroups` 的输出结构符合预期（有测试数据时）。

- [ ] **Step 5: Commit**

```bash
git add src/pages/weekReview.vue
git commit -m "feat(weekReview): add buildDayEntries computed for day view aggregation"
```

---

## Task 3: 新增 dayHours 状态、拖拽逻辑和 buildDayJsonData

**Files:**
- Modify: `src/pages/weekReview.vue`

### 目标

- `dayHours`：响应式 Map，`key = entry.key`，`value = hours (number)`，初始值由 `dayGroups` 变化时自动重置
- 拖拽函数 `startHoursDrag(event, entryKey, dayTs)`：左右拖动圆形把手调整工时
- `buildDayJsonData()`：按 entry.key 读取 `dayHours`，生成 JSON 数组
- `copyToClipboard` 中按天视图改为调用 `buildDayJsonData()`

### 默认工时计算

每天默认工时：`Math.max(0.5, Math.floor((8 / count) * 2) / 2)`，其中 `count` 为当天条目数。

- [ ] **Step 1: 新增 dayHours ref 和初始化逻辑**

```js
const dayHours = ref(new Map()); // key: entry.key, value: hours

// 当 dayGroups 变化时（切换到按天视图或数据更新），重新计算默认工时
watch(
  dayGroups,
  (groups) => {
    const newMap = new Map();
    for (const group of groups) {
      const count = group.entries.length;
      const defaultHours = Math.max(0.5, Math.floor((8 / count) * 2) / 2);
      for (const entry of group.entries) {
        // 保留用户已拖拽过的值
        newMap.set(entry.key, dayHours.value.get(entry.key) ?? defaultHours);
      }
    }
    dayHours.value = newMap;
  },
  { immediate: true }
);
```

- [ ] **Step 2: 新增 `dayDayTotal` 辅助函数**

```js
function getDayTotal(group) {
  return group.entries.reduce((sum, e) => sum + (dayHours.value.get(e.key) ?? 0), 0);
}
```

- [ ] **Step 3: 新增拖拽函数 `startHoursDrag`**

参考现有 `startDragProgress` 的模式（window mousemove/mouseup + touch 事件）：

```js
function startHoursDrag(event, entryKey) {
  event.stopPropagation();
  event.preventDefault();
  const barEl = event.currentTarget;

  function onMove(e) {
    const rect = barEl.getBoundingClientRect();
    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    // 比例映射到 0.5~24h，步进 0.5
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
  window.addEventListener("touchmove", onMove, { passive: true });
  window.addEventListener("touchend", onUp);
  window.addEventListener("touchcancel", onUp);
}
```

注意：`barEl` 是进度条 DOM 元素的引用，在 template 中通过 `@mousedown="startHoursDrag($event, entry.key, $el)"` 传入（见 Task 4）。

- [ ] **Step 4: 新增 `buildDayJsonData` 函数**

```js
function buildDayJsonData() {
  const result = [];
  for (const group of dayGroups.value) {
    const dayEnd = group.dayTs + 86400000 - 1; // 当天 23:59:59.999
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
```

- [ ] **Step 5: 更新 copyToClipboard 中按天视图分支**

将 Task 1 Step 2 中的临时 `buildJsonData()` 调用替换为：

```js
if (viewMode.value === "day") {
  content = JSON.stringify(buildDayJsonData(), null, 2);
}
```

- [ ] **Step 6: Commit**

```bash
git add src/pages/weekReview.vue
git commit -m "feat(weekReview): add day hours state, drag logic, and day JSON builder"
```

---

## Task 4: 渲染按天视图 Template 和样式

**Files:**
- Modify: `src/pages/weekReview.vue`

### 目标

在 `<template>` 中，根据 `viewMode` 条件渲染：
- `viewMode === 'project'`：现有 `<div class="list">` 卡片视图（保持不变）
- `viewMode === 'day'`：新增按天区块列表

- [ ] **Step 1: 包裹现有列表为 v-if**

将现有：
```html
<div v-if="reviewTopics.length === 0" class="empty">本周还没有完成的子任务</div>
<div v-else class="list"> ... </div>
```

替换为：
```html
<template v-if="viewMode === 'project'">
  <div v-if="reviewTopics.length === 0" class="empty">本周还没有完成的子任务</div>
  <div v-else class="list">
    <!-- 现有 topic-card 内容，保持不变 -->
  </div>
</template>

<template v-else>
  <div v-if="dayGroups.length === 0" class="empty">本周还没有完成的子任务</div>
  <div v-else class="list day-list">
    <!-- 按天视图，见下 -->
  </div>
</template>
```

- [ ] **Step 2: 实现按天视图 Template**

按天视图内容（替换上方 `<!-- 按天视图，见下 -->` 注释）：

```html
<div v-for="group in dayGroups" :key="group.dayTs" class="day-block">
  <div class="day-header">
    <span class="day-name">{{ group.dayLabel }}</span>
    <span class="day-date">{{ group.dateLabel }}</span>
    <span class="day-total" :class="{ over: getDayTotal(group) > 8 }">
      {{ getDayTotal(group) }}h
    </span>
  </div>
  <div v-for="entry in group.entries" :key="entry.key" class="day-task-row">
    <span class="day-task-status" :class="entry.isDone ? 'done' : 'progress'">
      {{ entry.isDone ? '✓' : '…' }}
    </span>
    <div class="day-task-info">
      <span class="day-task-topic">{{ entry.topicTitle }}</span>
      <span class="day-task-text">{{ entry.itemText }}</span>
    </div>
    <div
      class="hours-bar"
      @mousedown.stop.prevent="startHoursDrag($event, entry.key, $el)"
      @touchstart.stop.prevent="startHoursDrag($event, entry.key, $el)"
    >
      <div
        class="hours-bar-fill"
        :style="{ width: ((dayHours.get(entry.key) ?? 0) / 24 * 100) + '%' }"
      ></div>
      <div class="hours-bar-knob"></div>
    </div>
    <span class="hours-val" :class="{ over: getDayTotal(group) > 8 && (dayHours.get(entry.key) ?? 0) > 0 }">
      {{ dayHours.get(entry.key) ?? 0 }}h
    </span>
  </div>
</div>
```

注意：`@mousedown` 中的 `$el` 指向 `.hours-bar` 元素本身。在 Vue 3 template 中，事件处理内无法直接拿到 `$el`，需要用 `ref` 或改为在 handler 中从 `event.currentTarget` 获取。修正方案：

```html
<div
  class="hours-bar"
  @mousedown.stop.prevent="startHoursDrag($event, entry.key)"
  @touchstart.stop.prevent="startHoursDrag($event, entry.key)"
>
```

并在 `startHoursDrag` 中从 `event.currentTarget` 获取 `barEl`：

```js
function startHoursDrag(event, entryKey) {
  event.stopPropagation();
  event.preventDefault();
  const barEl = event.currentTarget;
  // 其余代码不变
  ...
}
```

- [ ] **Step 3: 新增按天视图 CSS**

在 `<style scoped>` 末尾追加，同时将现有 `.copy-btn` 的 `margin-left: auto` 删除（`margin-left: auto` 已移到 `.seg-control`）：

```css
/* 移除旧的 margin-left: auto（在新布局中 seg-control 负责推右） */
.copy-btn {
  /* 保留其他样式，删除 margin-left: auto */
  background: #1a1a1a;
  color: white;
  border: none;
  border-radius: 10px;
  padding: 6px 14px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}
```

```css
/* ===== 按天视图 ===== */
.day-block {
  flex-shrink: 0;
}

.day-header {
  display: flex;
  align-items: baseline;
  gap: 6px;
  padding: 4px 2px 6px;
}

.day-name {
  font-size: 14px;
  font-weight: 700;
  color: #1a1a1a;
}

.day-date {
  font-size: 11px;
  color: #bbb;
  flex: 1;
}

.day-total {
  font-size: 11px;
  font-weight: 600;
  color: #666;
  background: rgba(0, 0, 0, 0.06);
  border-radius: 6px;
  padding: 2px 8px;
}

.day-total.over {
  color: #e53e3e;
  background: #fff0f0;
}

.day-task-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  background: rgba(255, 255, 255, 0.7);
  border-radius: 10px;
  margin-bottom: 4px;
}

.day-task-status {
  font-size: 13px;
  flex-shrink: 0;
  width: 16px;
}

.day-task-status.done {
  color: #38a169;
}

.day-task-status.progress {
  color: #d97706;
}

.day-task-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.day-task-topic {
  font-size: 10px;
  color: #bbb;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.day-task-text {
  font-size: 12px;
  color: #444;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hours-bar {
  width: 52px;
  height: 6px;
  background: #eee;
  border-radius: 3px;
  position: relative;
  cursor: ew-resize;
  flex-shrink: 0;
}

.hours-bar-fill {
  height: 100%;
  border-radius: 3px;
  background: linear-gradient(90deg, #a7f3d0, #34d399);
  pointer-events: none;
}

.hours-bar-knob {
  position: absolute;
  right: -5px;
  top: 50%;
  transform: translateY(-50%);
  width: 12px;
  height: 12px;
  background: white;
  border-radius: 50%;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
  pointer-events: none;
}

.hours-val {
  font-size: 12px;
  font-weight: 600;
  color: #1a1a1a;
  min-width: 26px;
  text-align: right;
  flex-shrink: 0;
}

.hours-val.over {
  color: #e53e3e;
}
```

- [ ] **Step 4: 测试完整按天视图交互**

运行 `pnpm tauri dev`，验证：
1. 切换到「按天」视图后，按天区块正确出现
2. 每天任务默认工时正确均分（3个任务各 2.5h 等）
3. 拖拽 `.hours-bar` 区域，工时数值实时更新
4. 总工时超过 8h 时，`day-total` 和对应 `hours-val` 变红
5. 切回「项目」视图，现有卡片列表正常渲染
6. 按天视图点击「复制」，粘贴后验证 JSON 结构中 `duration` 为当天时间戳区间、`plannedHours` 为拖拽值

- [ ] **Step 5: Commit**

```bash
git add src/pages/weekReview.vue
git commit -m "feat(weekReview): render day view template with hours drag interaction"
```

---

## Task 5: 收尾检查与最终 commit

**Files:**
- Modify: `src/pages/weekReview.vue`

- [ ] **Step 1: 清理残留代码**

检查并删除：
- 已无引用的 `copyFormat` 变量（Task 1 已删，确认无遗漏）
- 旧的 format-switch CSS 类（Task 1 已删，确认无遗漏）
- 旧的 `buildJsonData` 函数（现在按天视图用 `buildDayJsonData`，项目视图不需要 JSON，可以删除）

- [ ] **Step 2: 边界情况验证**

- 本周无任何完成/进度任务时，两种视图均显示「本周还没有完成的子任务」
- 某任务 `progressAt` 存在但值为 null（旧数据）：按天视图不显示该条目
- 某任务 `doneAt` 和 `progressAt` 在同一天：只生成一条「已完成」条目，不重复

- [ ] **Step 3: 最终 commit**

```bash
git add src/pages/weekReview.vue
git commit -m "feat(weekReview): complete day view with drag hours and JSON export"
```
