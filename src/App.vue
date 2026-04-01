<script setup>
import { ref, computed, watch, onMounted } from "vue";
import { load } from "@tauri-apps/plugin-store";
import { getCurrentWindow, PhysicalPosition, PhysicalSize } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import EditIcon from '~icons/fluent/edit-12-filled';
import CloseMdIcon from '~icons/ci/close-md';
import DailyReviewIcon from '~icons/line-md/clipboard-list';
import WeeklyReviewIcon from '~icons/line-md/clipboard-list-twotone';
import FolderOpen from '~icons/fa7-regular/folder-open';
import FolderClosed from '~icons/fa7-regular/folder-closed';
import RunningShoe from '~icons/game-icons/running-shoe';
import DailyReview from './pages/dailyReview.vue';
import WeekReview from './pages/weekReview.vue';


const topics = ref([]);
const newTopicText = ref("");
const newItemText = ref({});
const collapsed = ref(false);
const contentVisible = ref(true);
const animating = ref(false);
const editingItemId = ref(null);
const editingItemText = ref("");
const currentPage = ref("main");
const slideDirection = ref("forward"); // forward: 主页→子页, backward: 子页→主页

function navigateTo(page) {
  slideDirection.value = "forward";
  currentPage.value = page;
}

function navigateBack() {
  slideDirection.value = "backward";
  currentPage.value = "main";
}

const priorityOrder = { high: 0, medium: 1, low: 2 };
const priorityLabel = { high: "高", medium: "中", low: "低" };

let store = null;
const FULL_WIDTH = 400;
const COLLAPSED_SIZE = Math.round((36 + 16) * (window.devicePixelRatio || 1));
const STEPS = 20;
const DURATION = 200; // ms

const collapseBtn = ref(null);

onMounted(async () => {
  store = await load("topics.json", { autoSave: true });
  const saved = await store.get("topics");
  if (saved) {
    saved.forEach(topic =>
      topic.items.forEach(item => {
        if (item.progress === undefined) item.progress = item.done ? 100 : 0;
      })
    );
    topics.value = saved;
  }
});

watch(topics, async (val) => {
  if (store) await store.set("topics", val);
}, { deep: true });

let savedSize = null;
let savedPos = null;

async function toggleCollapse() {
  if (animating.value) return;
  animating.value = true;
  const isCollapsing = !collapsed.value;
  try {
    const win = getCurrentWindow();
    const [outerSize, outerPos] = await Promise.all([win.outerSize(), win.outerPosition()]);
    const currentWidth = outerSize.width;
    const currentHeight = outerSize.height;
    const currentX = outerPos.x;
    const currentY = outerPos.y;

    let targetWidth, targetHeight, targetX, targetY;

    if (isCollapsing) {
      savedSize = { width: currentWidth, height: currentHeight };

      const scale = window.devicePixelRatio || 1;
      const btn = collapseBtn.value;
      if (btn) {
        const rect = btn.getBoundingClientRect();
        const paddingTop = rect.top;
        const paddingRight = window.innerWidth - rect.right;
        const pad = Math.min(paddingTop, paddingRight);
        const size = Math.round((pad + Math.max(rect.width, rect.height) + pad) * scale);
        targetWidth = size;
        targetHeight = size;
      } else {
        targetWidth = COLLAPSED_SIZE;
        targetHeight = COLLAPSED_SIZE;
      }
      targetX = currentX + currentWidth - targetWidth;
      targetY = currentY;

      // 隐藏内容但不切换icon
      contentVisible.value = false;
      await new Promise((r) => requestAnimationFrame(r));
    } else {
      targetWidth = savedSize ? savedSize.width : FULL_WIDTH;
      targetHeight = savedSize ? savedSize.height : currentHeight;
      targetX = currentX + currentWidth - targetWidth;
      targetY = currentY;
    }

    const diffW = targetWidth - currentWidth;
    const diffH = targetHeight - currentHeight;
    const diffX = targetX - currentX;
    const diffY = targetY - currentY;

    await new Promise((resolve) => {
      const startTime = performance.now();
      function frame(now) {
        const t = Math.min((now - startTime) / DURATION, 1);
        const eased = 1 - Math.pow(1 - t, 3);
        const w = Math.round(currentWidth + diffW * eased);
        const h = Math.round(currentHeight + diffH * eased);
        const x = Math.round(currentX + diffX * eased);
        const y = Math.round(currentY + diffY * eased);
        Promise.all([
          win.setPosition(new PhysicalPosition(x, y)),
          win.setSize(new PhysicalSize(w, h)),
        ]).then(() => {
          if (t < 1) requestAnimationFrame(frame);
          else resolve();
        });
      }
      requestAnimationFrame(frame);
    });

    // 动画结束后切换icon和内容
    collapsed.value = isCollapsing;
    if (!isCollapsing) contentVisible.value = true;
  } catch (e) {
    console.error("toggleCollapse error:", e);
  } finally {
    animating.value = false;
  }
}

function isTopicDone(topic) {
  return topic.items.length > 0 && topic.items.every((i) => i.done);
}

const oneWeekAgo = Date.now() - 7 * 24 * 60 * 60 * 1000;

const sortedTopics = computed(() => {
  const visible = topics.value.filter((t) => {
    const createdAt = t.createdAt || 0;
    if (isTopicDone(t) && createdAt < oneWeekAgo) return false;
    return true;
  });
  return [
    ...visible.filter((t) => !isTopicDone(t)),
    ...visible.filter((t) => isTopicDone(t)),
  ];
});

function addTopic() {
  const text = newTopicText.value.trim();
  if (!text) return;
  topics.value.push({ id: Date.now(), createdAt: Date.now(), title: text, priority: "medium", expanded: true, items: [] });
  newTopicText.value = "";
}

function removeTopic(id) {
  topics.value = topics.value.filter((t) => t.id !== id);
}

function cyclePriority(topic) {
  const order = ["high", "medium", "low"];
  topic.priority = order[(order.indexOf(topic.priority) + 1) % order.length];

  const idx = topics.value.indexOf(topic);
  topics.value.splice(idx, 1);

  if (topic.priority === "high") {
    topics.value.unshift(topic);
  } else if (topic.priority === "low") {
    topics.value.push(topic);
  } else {
    const lastHighIdx = topics.value.reduce((acc, t, i) => t.priority === "high" ? i : acc, -1);
    topics.value.splice(lastHighIdx + 1, 0, topic);
  }
}

function toggleExpanded(topic) {
  topic.expanded = !topic.expanded;
}

function addItem(topic) {
  const text = (newItemText.value[topic.id] || "").trim();
  if (!text) return;
  topic.items.push({ id: Date.now(), text, done: false, progress: 0 });
  newItemText.value[topic.id] = "";
}

function toggleItem(item, topic) {
  item.done = !item.done;
  item.doneAt = item.done ? Date.now() : null;
  if (item.done) {
    item.progress = 100;
  }
  if (topic && isTopicDone(topic)) {
    topic.expanded = false;
  }
}

function removeItem(topic, itemId) {
  topic.items = topic.items.filter((i) => i.id !== itemId);
}

function startEditItem(item) {
  editingItemId.value = item.id;
  editingItemText.value = item.text;
}

function saveEditItem(item) {
  const text = editingItemText.value.trim();
  if (text) item.text = text;
  editingItemId.value = null;
}

function sortedItems(topic) {
  return [...topic.items.filter((i) => !i.done), ...topic.items.filter((i) => i.done)];
}

function formatTime(id) {
  const ts = typeof id === "number" ? id : Date.now();
  const d = new Date(ts);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const itemDay = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  const diffDays = Math.round((today - itemDay) / 86400000);
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  if (diffDays === 0) return `今天 ${hh}:${mm}`;
  if (diffDays === 1) return `昨天 ${hh}:${mm}`;
  if (diffDays === 2) return `前天 ${hh}:${mm}`;
  const mo = String(d.getMonth() + 1).padStart(2, "0");
  const da = String(d.getDate()).padStart(2, "0");
  return `${mo}-${da} ${hh}:${mm}`;
}

function sortByPriority() {
  topics.value.sort((a, b) => priorityOrder[a.priority] - priorityOrder[b.priority]);
}

function startDragProgress(event, item) {
  // 阻止冒泡，避免触发 toggleItem
  event.stopPropagation();
  const el = event.currentTarget.closest('.item');
  if (!el) return;

  const touchMoveOpts = { passive: true };

  // 拖拽时禁用 transition
  const fillEl = el.querySelector('.item-progress-fill');
  const shoeEl = el.querySelector('.item-shoe-float');
  if (fillEl) fillEl.style.transition = 'none';
  if (shoeEl) shoeEl.style.transition = 'none';

  function onMove(e) {
    const rect = el.getBoundingClientRect();
    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const pct = Math.round(((clientX - rect.left) / rect.width) * 100);
    item.progress = Math.max(0, Math.min(100, pct));
  }

  function onUp() {
    // 拖拽结束恢复 transition
    if (fillEl) fillEl.style.transition = '';
    if (shoeEl) shoeEl.style.transition = '';
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
    window.removeEventListener('touchmove', onMove, touchMoveOpts);
    window.removeEventListener('touchend', onUp);
    window.removeEventListener('touchcancel', onUp);
  }

  // 立即更新到点击位置
  onMove(event);

  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
  window.addEventListener('touchmove', onMove, touchMoveOpts);
  window.addEventListener('touchend', onUp);
  window.addEventListener('touchcancel', onUp);
}
</script>

<template>
  <div class="page-root">
    <Transition :name="slideDirection === 'forward' ? 'slide-left' : 'slide-right'">
      <DailyReview v-if="currentPage === 'dailyReview'" key="dailyReview" @back="navigateBack" />
      <WeekReview v-else-if="currentPage === 'weekReview'" key="weekReview" @back="navigateBack" />
      <div v-else key="main" class="app" :class="{ collapsed }">
        <div class="header">
          <span class="title" v-if="contentVisible">主题待办</span>
          <div class="header-actions">
            <button v-if="contentVisible" class="review-btn" @click="navigateTo('dailyReview')" title="今日完成">
              <DailyReviewIcon />
            </button>
            <button v-if="contentVisible" class="sort-btn" @click="navigateTo('weekReview')" title="本周完成">
              <WeeklyReviewIcon />
            </button>
            <button ref="collapseBtn" class="collapse-btn" @click="toggleCollapse" :title="collapsed ? '展开' : '收起'">
              <svg class="icon" aria-hidden="true">
                <use :xlink:href="collapsed ? '#icon-zhankai' : '#icon-shouqi1'"></use>
              </svg>
            </button>
          </div>
        </div>

        <template v-if="contentVisible">
          <div class="input-row">
            <input v-model="newTopicText" placeholder="新建主题..." @keydown.enter="addTopic" />
            <button class="add-btn" @click="addTopic">+</button>
          </div>

          <div class="list">
            <div v-for="topic in sortedTopics" :key="topic.id" class="topic-card"
              :class="[topic.priority, { done: isTopicDone(topic) }]">
              <div class="topic-header">
                <button class="expand-btn" @click="toggleExpanded(topic)">
                  <FolderOpen v-if="!topic.expanded" />
                  <FolderClosed v-else />
                </button>
                <span class="topic-title" :class="{ strikethrough: isTopicDone(topic) }">
                  {{ topic.title }}
                </span>
                <span class="progress">{{topic.items.filter(i => i.done).length}}/{{ topic.items.length }}</span>
                <button class="priority-btn" :class="topic.priority" @click="cyclePriority(topic)">
                  {{ priorityLabel[topic.priority] }}
                </button>
                <button class="remove-btn" @click="removeTopic(topic.id)">×</button>
              </div>

              <div v-if="topic.expanded" class="items">
                <div v-for="item in sortedItems(topic)" :key="item.id" class="item" :class="{ done: item.done }">
                  <div
                    class="item-progress-fill"
                    :style="{ width: (item.progress || 0) + '%' }"
                  ></div>
                  <span class="checkbox" @click="toggleItem(item, topic)">{{ item.done ? '✓' : '○' }}</span>
                  <template v-if="editingItemId === item.id">
                    <div class="item-content-layer">
                      <input class="item-edit-input" v-model="editingItemText" @keydown.enter="saveEditItem(item)"
                        @keydown.esc="editingItemId = null" @blur="saveEditItem(item)" :ref="el => el && el.focus()" />
                    </div>
                  </template>
                  <template v-else>
                    <div class="item-content-layer">
                      <span class="item-text" @click="toggleItem(item, topic)">{{ item.text }}</span>
                      <span class="item-time">{{ formatTime(item.id) }}</span>
                      <button class="edit-btn" @click="startEditItem(item)">
                        <EditIcon />
                      </button>
                    </div>
                  </template>
                  <button class="remove-btn" @click="removeItem(topic, item.id)">
                    <CloseMdIcon />
                  </button>
                  <div
                    class="item-shoe-float"
                    :class="{ 'shoe-hidden': !item.progress }"
                    :style="{ left: 'calc(' + (item.progress || 0) + '% - 10px)' }"
                    @mousedown.stop="startDragProgress($event, item)"
                    @touchstart.stop.prevent="startDragProgress($event, item)"
                  >
                    <RunningShoe class="shoe-svg" />
                    <span class="shoe-pct">{{ item.progress || 0 }}%</span>
                  </div>
                </div>
                <div class="item-input-row">
                  <input v-model="newItemText[topic.id]" placeholder="添加待办..." @keydown.enter="addItem(topic)" />
                  <button class="item-add-btn" :class="topic.priority" @click="addItem(topic)">+</button>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>
    </Transition>
  </div>
</template>
<style>
.icon {
  width: 18px;
  height: 18px;
  fill: currentColor;
  color: #666;
  overflow: hidden;
}

.page-root {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.slide-left-enter-active,
.slide-left-leave-active,
.slide-right-enter-active,
.slide-right-leave-active {
  transition: transform 0.22s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.22s;
  position: absolute;
  width: 100%;
  height: 100%;
  top: 0;
  left: 0;
}

.slide-left-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.slide-left-enter-to {
  transform: translateX(0);
  opacity: 1;
}

.slide-left-leave-from {
  transform: translateX(0);
  opacity: 1;
}

.slide-left-leave-to {
  transform: translateX(-100%);
  opacity: 0;
}

.slide-right-enter-from {
  transform: translateX(-100%);
  opacity: 0;
}

.slide-right-enter-to {
  transform: translateX(0);
  opacity: 1;
}

.slide-right-leave-from {
  transform: translateX(0);
  opacity: 1;
}

.slide-right-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 20px 16px;
  gap: 12px;
  background: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(12px);
  border-radius: 16px;
}

.app.collapsed {
  background: transparent;
  border-radius: 50%;
  box-shadow: none;
  overflow: hidden;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px;
}

.title {
  font-size: 17px;
  font-weight: 600;
  color: #1a1a1a;
}

.sort-btn {
  background: white;
  border: none;
  border-radius: 10px;
  width: 36px;
  height: 36px;
  font-size: 16px;
  cursor: pointer;
  color: #666;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.review-btn {
  background: white;
  border: none;
  border-radius: 10px;
  width: 36px;
  height: 36px;
  font-size: 16px;
  cursor: pointer;
  color: #666;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.input-row {
  display: flex;
  gap: 8px;
}

.input-row input {
  flex: 1;
  padding: 11px 14px;
  border: none;
  border-radius: 12px;
  background: white;
  font-size: 15px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  outline: none;
}

.add-btn {
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 12px;
  background: #e0e0e0;
  color: #666;
  font-size: 22px;
  cursor: pointer;
  line-height: 1;
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
  transition: box-shadow 0.2s, opacity 0.2s;
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

.topic-card.done {
  opacity: 0.55;
  background: #f3f3f3;
}

.topic-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 13px 14px;
}

.expand-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: #999;
  padding: 0;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.expand-btn svg {
  width: 16px;
  height: 16px;
  display: block;
}

.topic-title {
  flex: 1;
  font-size: 15px;
  font-weight: 600;
  color: #1a1a1a;
}

.topic-title.strikethrough {
  text-decoration: line-through;
  color: #aaa;
}

.progress {
  font-size: 13px;
  color: #aaa;
}

.priority-btn {
  border: none;
  border-radius: 8px;
  padding: 3px 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.priority-btn.high {
  background: #ffe0e0;
  color: #e53e3e;
}

.priority-btn.medium {
  background: #fef3c7;
  color: #d97706;
}

.priority-btn.low {
  background: #dcfce7;
  color: #38a169;
}

.remove-btn {
  background: none;
  border: none;
  color: #ccc;
  font-size: 18px;
  cursor: pointer;
  opacity: 0;
  transition: color 0.2s, opacity 0.2s;
  padding: 0 2px;
  line-height: 1;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  align-self: center;
}

.topic-header:hover .remove-btn,
.item:hover .remove-btn {
  opacity: 1;
}

.item .remove-btn {
  align-self: flex-start;
  width: 22px;
  height: 22px;
  padding: 0;
  font-size: 16px;
}

.remove-btn:hover {
  opacity: 1;
  color: #e53e3e;
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
  transition: opacity 0.2s;
}

.item.done {
  opacity: 0.5;
}

.item.done .item-text {
  text-decoration: line-through;
  color: #aaa;
}

.checkbox {
  font-size: 15px;
  color: #aaa;
  cursor: pointer;
  user-select: none;
  width: 18px;
  flex-shrink: 0;
  align-self: flex-start;
  padding-top: 2px;
}

.item.done .checkbox {
  color: #38a169;
}

.item-text {
  flex: 1;
  font-size: 14px;
  color: #333;
  cursor: pointer;
  user-select: none;
  line-height: 1.4;
}

.item-time {
  font-size: 11px;
  color: #bbb;
  white-space: nowrap;
  align-self: flex-start;
  line-height: 19.6px;
}

.edit-btn {
  background: none;
  border: none;
  color: #ccc;
  cursor: pointer;
  opacity: 0;
  transition: color 0.2s, opacity 0.2s;
  padding: 0 2px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  align-self: flex-start;
  width: 22px;
  height: 22px;
}

.item:hover .edit-btn {
  opacity: 1;
}

.edit-btn svg {
  width: 14px;
  height: 14px;
  display: block;
}

.edit-btn:hover {
  color: #888;
}

.item-edit-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 14px;
  color: #333;
  padding: 0;
}

.item-input-row {
  display: flex;
  gap: 6px;
  margin-top: 2px;
}

.item-input-row input {
  flex: 1;
  padding: 7px 12px;
  border: none;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.8);
  font-size: 14px;
  outline: none;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.item-add-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 10px;
  font-size: 20px;
  cursor: pointer;
  line-height: 1;
  background: rgba(0, 0, 0, 0.06);
  color: #666;
}

.item-add-btn.high {
  background: #ffd0d0;
  color: #e53e3e;
}

.item-add-btn.medium {
  background: #fde68a;
  color: #d97706;
}

.item-add-btn.low {
  background: #bbf7d0;
  color: #38a169;
}

.header-actions {
  display: flex;
  gap: 6px;
}

.collapse-btn {
  background: white;
  border: none;
  border-radius: 10px;
  width: 36px;
  height: 36px;
  font-size: 18px;
  cursor: pointer;
  color: #666;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  transition: box-shadow 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.collapse-btn:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.app.collapsed .collapse-btn {
  border-radius: 10px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
  backdrop-filter: blur(8px);
}

/* ===== Item Progress Bar ===== */

/* item 需要 relative 定位以容纳绝对定位子元素 */
.item {
  position: relative;
  overflow: hidden;
}

/* 进度填充层：绿色渐变背景，覆盖在 item 背景之上、内容之下 */
.item-progress-fill {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  background: linear-gradient(90deg, rgba(187, 247, 208, 0.75) 0%, rgba(110, 231, 183, 0.45) 80%, transparent 100%);
  border-radius: 10px 0 0 10px;
  transition: width 0.55s cubic-bezier(0.4, 0, 0.2, 1);
  pointer-events: none;
  z-index: 0;
}

/* 当进度为 100% 时，右侧圆角也填满 */
.item.done .item-progress-fill,
.item-progress-fill[style*="width: 100"] {
  border-radius: 10px;
}

/* item 内所有直接内容层级高于进度条 */
.item > .checkbox,
.item > .item-text,
.item > .item-time,
.item > .edit-btn,
.item > .remove-btn,
.item > template {
  position: relative;
  z-index: 1;
}

/* RunningShoe 浮层：绝对定位在进度边界 */
.item-shoe-float {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: ew-resize;
  z-index: 2;
  transition: left 0.55s cubic-bezier(0.4, 0, 0.2, 1);
  pointer-events: auto;
  user-select: none;
}

/* 拖拽时禁用过渡动画（避免拖拽延迟感） */
.item-shoe-float.dragging {
  transition: none;
}

.item-shoe-float .shoe-svg {
  width: 14px;
  height: 14px;
  color: #059669;
  filter: drop-shadow(0 1px 2px rgba(0,0,0,0.2));
}

.item-shoe-float .shoe-pct {
  font-size: 9px;
  font-weight: 800;
  color: #065f46;
  line-height: 1;
  white-space: nowrap;
  background: rgba(255,255,255,0.8);
  border-radius: 4px;
  padding: 1px 3px;
  margin-top: 1px;
}

/* 进度为 0 时隐藏图标（用 class binding 而非属性选择器，避免空格不匹配） */
.item-shoe-float.shoe-hidden {
  opacity: 0;
  pointer-events: none;
}

.item-content-layer {
  display: contents;
  position: relative;
  z-index: 1;
}
</style>