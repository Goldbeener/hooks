# Item Progress Bar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为每个子任务（item）添加进度功能：整行作为进度条，绿色渐变背景从左填充，RunningShoe 图标浮在进度边界可拖动调整进度，勾选完成时动画充满全行。

**Architecture:** 在 `src/App.vue` 中为每个 item 数据对象增加 `progress` 字段（0-100 整数），在 item 元素上用 `position: relative` + 绝对定位的渐变填充层实现进度背景，RunningShoe 图标通过 `mousedown` 拖拽事件计算相对位置更新进度值，`toggleItem` 完成时将 progress 设为 100 并触发 CSS transition 动画充满整行。

**Tech Stack:** Vue 3 Composition API, CSS transition/transform, pointer events (mousedown/mousemove/mouseup), Iconify RunningShoe icon（已在项目中引入）

---

## File Map

| File | 操作 | 说明 |
|------|------|------|
| `src/App.vue` | Modify | 唯一需要修改的文件：数据模型、逻辑函数、模板、样式 |

---

### Task 1: 扩展 item 数据模型，增加 progress 字段

**Files:**
- Modify: `src/App.vue` — `addItem` 函数、`toggleItem` 函数

- [ ] **Step 1: 在 `addItem` 函数中为新 item 添加 `progress: 0`**

找到（约第 194 行）：
```js
topic.items.push({ id: Date.now(), text, done: false });
```
改为：
```js
topic.items.push({ id: Date.now(), text, done: false, progress: 0 });
```

- [ ] **Step 2: 修改 `toggleItem` 函数，勾选完成时将 progress 设为 100，取消完成时保留 progress 不变**

找到（约第 198 行）：
```js
function toggleItem(item, topic) {
  item.done = !item.done;
  item.doneAt = item.done ? Date.now() : null;
  if (topic && isTopicDone(topic)) {
    topic.expanded = false;
  }
}
```
改为：
```js
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
```

- [ ] **Step 3: 手动验证（无测试框架）**

运行 `pnpm tauri dev`，添加一个子任务，确认数据结构正确（打开 devtools 查看 topics.value）。

- [ ] **Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat: add progress field to item data model"
```

---

### Task 2: 添加拖拽设置进度的逻辑函数

**Files:**
- Modify: `src/App.vue` — `<script setup>` 区域，在 `sortByPriority` 之后添加新函数

- [ ] **Step 1: 在 `</script>` 之前（约第 244 行后）添加 `startDragProgress` 函数**

```js
function startDragProgress(event, item) {
  // 阻止冒泡，避免触发 toggleItem
  event.stopPropagation();
  const el = event.currentTarget.closest('.item');
  if (!el) return;

  function onMove(e) {
    const rect = el.getBoundingClientRect();
    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const pct = Math.round(((clientX - rect.left) / rect.width) * 100);
    item.progress = Math.max(0, Math.min(100, pct));
  }

  function onUp() {
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
    window.removeEventListener('touchmove', onMove);
    window.removeEventListener('touchend', onUp);
  }

  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
  window.addEventListener('touchmove', onMove, { passive: true });
  window.addEventListener('touchend', onUp);
}
```

- [ ] **Step 2: 运行开发服务器，确认无语法错误**

```bash
pnpm tauri dev
```
预期：控制台无报错，页面正常渲染。

- [ ] **Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat: add drag progress handler function"
```

---

### Task 3: 更新 item 模板，加入进度背景层和 RunningShoe 图标

**Files:**
- Modify: `src/App.vue` — `<template>` 中 `.item` 的 div 块（约第 295-311 行）

- [ ] **Step 1: 找到现有 item div，在其内部最前面插入进度背景层**

找到（约第 295 行）：
```html
<div v-for="item in sortedItems(topic)" :key="item.id" class="item" :class="{ done: item.done }">
```

在该 `<div>` 开始标签后、`<span class="checkbox">` 之前，插入：
```html
  <div
    class="item-progress-fill"
    :style="{ width: (item.progress || 0) + '%' }"
  ></div>
```

- [ ] **Step 2: 在模板中添加 RunningShoe 浮层（在 `.item` div 内，作为最后一个子元素，紧接在 remove-btn 之后）**

在 `<button class="remove-btn" @click="removeItem(topic, item.id)">` 那个按钮的关闭标签 `</button>` 之后、外层 `.item` div 的 `</div>` 之前，插入：

```html
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
```

注意：`RunningShoe` 已在第 12 行 import，直接使用即可。

- [ ] **Step 3: 运行并验证模板渲染无报错**

```bash
pnpm tauri dev
```
预期：item 列表正常渲染，无 Vue 警告。

- [ ] **Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat: add progress fill layer and shoe icon to item template"
```

---

### Task 4: 添加进度条和图标的 CSS 样式

**Files:**
- Modify: `src/App.vue` — `<style scoped>` 区块末尾

- [ ] **Step 1: 在 `<style scoped>` 最后的 `.app.collapsed .collapse-btn {}` 规则之后追加以下样式**

```css
/* ===== Item Progress Bar ===== */

/* item 需要 relative 定位以容纳绝对定位子元素 */
.item {
  position: relative;
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
```

- [ ] **Step 2: 验证样式生效**

```bash
pnpm tauri dev
```
预期：item 行显示绿色渐变进度背景，RunningShoe 图标出现在进度边界处，图标下方显示百分比。

- [ ] **Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat: add progress bar and shoe icon styles"
```

---

### Task 5: 确保内容层 z-index 正确（template 子元素）

**Files:**
- Modify: `src/App.vue` — template 中 item 的 `<template v-if>` 和 `<template v-else>` 包裹层

由于 `<template>` 标签不会渲染为 DOM 元素，其内部的 `<input>` 和 `<span>` 不会自动获得 `position: relative; z-index: 1`。需要给编辑态和显示态的内容加上包裹 `<div>`。

- [ ] **Step 1: 给编辑态 template 加包裹 div**

找到（约第 297 行）：
```html
<template v-if="editingItemId === item.id">
  <input class="item-edit-input" ... />
</template>
```
改为：
```html
<template v-if="editingItemId === item.id">
  <div class="item-content-layer">
    <input class="item-edit-input" v-model="editingItemText" @keydown.enter="saveEditItem(item)"
      @keydown.esc="editingItemId = null" @blur="saveEditItem(item)" :ref="el => el && el.focus()" />
  </div>
</template>
```

- [ ] **Step 2: 给显示态 template 加包裹 div**

找到（约第 301 行）：
```html
<template v-else>
  <span class="item-text" @click="toggleItem(item, topic)">{{ item.text }}</span>
  <span class="item-time">{{ formatTime(item.id) }}</span>
  <button class="edit-btn" @click="startEditItem(item)">
    <EditIcon />
  </button>
</template>
```
改为：
```html
<template v-else>
  <div class="item-content-layer">
    <span class="item-text" @click="toggleItem(item, topic)">{{ item.text }}</span>
    <span class="item-time">{{ formatTime(item.id) }}</span>
    <button class="edit-btn" @click="startEditItem(item)">
      <EditIcon />
    </button>
  </div>
</template>
```

- [ ] **Step 3: 在 `<style scoped>` 中为 `.item-content-layer` 添加样式**

```css
.item-content-layer {
  display: contents; /* 不影响 flex 布局，仅作为层级容器 */
  position: relative;
  z-index: 1;
}
```

> 注意：`display: contents` 使该 div 本身不参与 flex 布局，其子元素直接成为 flex items，保持原有布局不变。

- [ ] **Step 4: 验证编辑、显示、删除功能均正常**

```bash
pnpm tauri dev
```
预期：点击编辑按钮可正常进入编辑态，文字点击可切换完成状态，删除按钮正常工作，进度条在内容下方。

- [ ] **Step 5: Commit**

```bash
git add src/App.vue
git commit -m "feat: wrap item content in layer for correct z-index stacking"
```

---

### Task 6: 拖拽时禁用 transition（流畅拖动）

**Files:**
- Modify: `src/App.vue` — `startDragProgress` 函数、`.item-shoe-float` 模板

拖拽过程中，`left` 和 `width` 的 CSS transition 会造成明显延迟感，需要在拖拽时动态禁用。

- [ ] **Step 1: 修改 `startDragProgress` 函数，拖拽时给图标和填充层加 class**

将 Task 2 中写的 `startDragProgress` 替换为：

```js
function startDragProgress(event, item) {
  event.stopPropagation();
  const el = event.currentTarget.closest('.item');
  if (!el) return;

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
    window.removeEventListener('touchmove', onMove);
    window.removeEventListener('touchend', onUp);
  }

  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
  window.addEventListener('touchmove', onMove, { passive: true });
  window.addEventListener('touchend', onUp);
}
```

- [ ] **Step 2: 测试拖拽流畅度**

```bash
pnpm tauri dev
```
预期：拖动 RunningShoe 图标时，进度实时更新无延迟；松开后，后续程序化进度变化（如点击完成）仍有平滑动画。

- [ ] **Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat: disable transition during drag for smooth progress update"
```

---

### Task 7: 端对端验证所有功能

- [ ] **Step 1: 运行应用**

```bash
pnpm tauri dev
```

- [ ] **Step 2: 验证清单**

| 功能 | 预期行为 |
|------|----------|
| 新建 item | progress 默认为 0，不显示图标（opacity 0） |
| 拖动 RunningShoe | 进度实时更新，背景绿色渐变扩展，图标跟随移动，无卡顿 |
| progress > 0 | RunningShoe 图标和百分比数字可见 |
| 勾选完成 | progress 动画充满至 100%，整行变绿，图标停在最右侧 |
| 取消完成 | done 恢复 false，progress 保持 100（不回退） |
| 数据持久化 | 关闭重开应用，进度数据从 topics.json 正确恢复 |
| 编辑 item 文字 | 进度条正常显示，编辑功能不受影响 |
| 删除 item | 正常删除，无遗留事件监听器 |

- [ ] **Step 3: 最终 commit**

```bash
git add src/App.vue
git commit -m "feat: complete item progress bar with drag interaction"
```

---

## 注意事项

- **现有 item 数据兼容**：旧数据中 item 没有 `progress` 字段，模板中统一使用 `item.progress || 0` 处理，无需迁移。
- **RunningShoe 图标**：已在 App.vue 第 12 行 `import RunningShoe from '~icons/game-icons/running-shoe'`，直接在模板中使用 `<RunningShoe />` 即可。
- **避免 z-index 问题**：进度填充层 z-index: 0，内容 z-index: 1，图标浮层 z-index: 2。
- **不修改 .done 的 opacity**：done item 已有 `opacity: 0.5`，进度满 100% 后视觉上仍偏暗，这是预期行为（与整体设计一致）。
