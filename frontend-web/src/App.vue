<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";

const scrollWrapperRef = ref<HTMLElement | null>(null);
const pageRef = ref<HTMLTextAreaElement | null>(null);

const text = ref<string>("");
const title = ref<string | null>(null);

type SaveState = { type: "unlinked" } | { type: "linked"; dirty: boolean };

const saveState = ref<SaveState>({ type: "unlinked" });

const isLinked = computed(() => saveState.value.type === "linked");
const isDirty = computed(
  () => saveState.value.type === "linked" && saveState.value.dirty,
);
const shouldWarnOnLeave = computed(
  () =>
    saveState.value.type === "unlinked" ||
    (saveState.value.type === "linked" && saveState.value.dirty),
);

const markLinked = (dirty: boolean) =>
  (saveState.value = { type: "linked", dirty });
const markUnlinked = () => (saveState.value = { type: "unlinked" });
const markDirty = () => {
  if (saveState.value.type === "linked") saveState.value.dirty = true;
};
const markSaved = () => {
  if (saveState.value.type === "linked") saveState.value.dirty = false;
};

const lastSavedContent = ref<string>("");
const isLoading = ref<boolean>(false);

const API_CONTENT_URL = "/api/content";
const API_STATUS_URL = "/api/status";

watch(text, (newText) => {
  if (!isLinked.value) return;
  if (newText === lastSavedContent.value) {
    markSaved();
  } else {
    markDirty();
  }
});

const confirmLeave = (event: BeforeUnloadEvent) => {
  if (!shouldWarnOnLeave.value) return;
  event.preventDefault();
  event.returnValue = "";
};

onMounted(async () => {
  window.addEventListener("beforeunload", confirmLeave);
  await loadContent();
  syncPageHeight();
});

onUnmounted(() => window.removeEventListener("beforeunload", confirmLeave));

const loadContent = async () => {
  if (isLoading.value) return;
  try {
    isLoading.value = true;
    const response = await fetch(API_CONTENT_URL);
    if (!response.ok) throw new Error("Load error");

    const data = await response.json();

    title.value = data.title;
    text.value = data.content;
    lastSavedContent.value = data.content;

    if (data.saved) {
      markLinked(false);
    } else {
      markUnlinked();
    }
  } catch (error) {
    console.error("Load error:", error);
    markUnlinked();
  } finally {
    isLoading.value = false;
  }
};

const handleSaveFile = async () => {
  if (isLoading.value) return;
  try {
    isLoading.value = true;
    const response = await fetch(API_CONTENT_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        content: text.value,
        title: title.value,
        saved: false,
      }),
    });

    if (!response.ok) throw new Error();

    const data = await response.json();
    text.value = data.content;
    title.value = data.title;
    lastSavedContent.value = data.content;

    if (data.saved) {
      markLinked(false);
    } else {
      markUnlinked();
    }
  } catch (error) {
    markUnlinked();
    alert("Error in saving files, check your backend state.");
  } finally {
    isLoading.value = false;
  }
};

const HEIGHT: number = 1123; // 96 dpi 下 A4 纸像素高度
const pageHeightPx = ref<number>(HEIGHT);

const syncPageHeight = () => {
  if (!pageRef.value) return;
  const scrollWrapper = scrollWrapperRef.value;
  const previousScrollTop = scrollWrapper?.scrollTop ?? 0;

  pageRef.value.style.height = "auto";
  pageHeightPx.value = Math.max(pageRef.value.scrollHeight, HEIGHT);
  pageRef.value.style.height = `${pageHeightPx.value}px`;

  if (scrollWrapper) {
    scrollWrapper.scrollTop = previousScrollTop;
  }
};

watch(title, () => {
  document.title = title.value ?? "Simply Writer";
});

const zoomLevel = ref<number>(1);

const changeZoomLevel = (delta: number) =>
  (zoomLevel.value = Number(
    Math.max(0.5, Math.min(3, zoomLevel.value + delta)).toFixed(2),
  ));

const handleCtrlWheel = (e: WheelEvent) => {
  e.stopPropagation();
  const delta = e.deltaY > 0 ? -0.25 : 0.25;
  changeZoomLevel(delta);
};

const resetZoomLevel = () => (zoomLevel.value = 1);

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === "Tab") {
    e.preventDefault();
    // 看到下面划线的方法没
    // 想要不用？那就写一长串 Range 和 Selection API 的东西吧，最后还不支持原生的撤销栈
    document.execCommand("insertText", false, "\t");
  }

  // 只是多按了下大写锁定，你猜怎么着
  if ((e.key === "s" || e.key === "S") && e.ctrlKey) {
    e.preventDefault();
    handleSaveFile();
  }
};

const handleInput = async () => {
  await nextTick();
  // 等待前端把输入的东西插进 DOM 后才开始计算真实高度
  syncPageHeight();
};

const saveTip = computed<string>(() => {
  if (saveState.value.type === "unlinked") {
    return "⚠ Unlinked";
  }
  if (saveState.value.dirty) {
    return "● Modified";
  } else {
    return "✓ Saved";
  }
});

const CHECK_LINK_MS = 1000 * 30; // 半分钟一次

const checkLink = async () => {
  try {
    const response = await fetch(API_STATUS_URL);
    if (!response.ok) {
      markUnlinked();
      return;
    }

    if (saveState.value.type === "linked" && !saveState.value.dirty) {
      await loadContent();
    }
  } catch (err) {
    markUnlinked();
  } finally {
    setTimeout(checkLink, CHECK_LINK_MS);
  }
};

onMounted(checkLink);
</script>

<template>
  <main class="viewport" @wheel.ctrl.prevent="handleCtrlWheel">
    <div class="scroll-wrapper" ref="scrollWrapperRef">
      <div
        class="zoom-container"
        :style="{
          width: `${793 * zoomLevel}px`,
          height: `${pageHeightPx * zoomLevel}px`,
        }"
      >
        <textarea
          ref="pageRef"
          class="page"
          @keydown="handleKeydown"
          @input="handleInput"
          spellcheck="false"
          v-model="text"
          :style="{ transform: `scale(${zoomLevel})` }"
        ></textarea>
      </div>
    </div>
    <div class="status-bar">
      <div class="status-left">
        <span
          class="save-indicator"
          :class="{
            modified: isDirty,
            unsafed: !isLinked,
          }"
          @click="handleSaveFile"
        >
          {{ saveTip }}
        </span>
      </div>
      <div class="status-right">
        <div class="zoom-controls">
          <span @click="changeZoomLevel(-0.25)">-</span>
          <span class="zoom-indicator" @click="resetZoomLevel">
            {{ Math.round(zoomLevel * 100) }}%
          </span>
          <span @click="changeZoomLevel(0.25)">+</span>
        </div>
        <span class="word-count">{{ text.length }} 个字</span>
      </div>
    </div>
  </main>
</template>

<style scoped>
::selection {
  color: #000;
  background: #d9d9d9;
}

.viewport {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: #f0f2f5;
  display: flex;
  flex-direction: column;
}

.init-prompt {
  background-color: white;
  border: 1px solid #e4e7ed;
  width: 24rem;
  height: 20rem;
  margin: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  border-radius: 0.5rem;
  box-shadow: 0 0.6rem 2rem rgba(0, 0, 0, 0.08);
}

.u-cannt-see-me {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  border: 0;
  clip: rect(0 0 0 0);
  overflow: hidden;
}

.init-divider {
  color: #909399;
  font-size: 0.9rem;
}

.scroll-wrapper {
  flex: 1;
  overflow: auto;
  padding: 3rem;
}

.zoom-container {
  margin: 0 auto;
  padding: 5rem 0;
  position: relative;
  display: flex;
  justify-content: center;
  min-height: 100%;
}

.page {
  transform-origin: top center;
  position: absolute;
  top: 0;
  tab-size: 4;
  width: 210mm;
  height: auto;
  min-height: 297mm;
  padding: 4rem;
  background-color: white;
  box-shadow: 0 0.2rem 1.5rem rgba(0, 0, 0, 0.1);
  outline: none;
  border: 1px solid #dcdfe6;
  word-break: break-all;
  overflow-wrap: anywhere;
  white-space: break-spaces;
  font-size: 15px;
  line-height: 1.6;
  resize: none;
  overflow: hidden;
  font-family: serif;
}

.status-bar {
  height: 1.75rem;
  background-color: #f5f5f5;
  border-top: 1px solid #dcdfe6;
  display: flex;
  align-items: center;
  justify-content: space-between;
  user-select: none;
  flex-shrink: 0;
}

.status-left {
  display: flex;
  align-items: center;
  height: 100%;
}

.status-right {
  display: flex;
  align-items: center;
  height: 100%;
}

.status-bar span {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 0.75rem;
  font-size: 0.75rem;
  color: #606266;
  transition: background 0.1s;
  cursor: pointer;
}

.status-bar span:hover {
  background: #e4e7ed;
}

.save-indicator {
  cursor: pointer;
  color: #67c23a;
  font-weight: 500;
}

.save-indicator.modified {
  color: #e6a23c;
}

.save-indicator.unsafed {
  color: #f56c6c;
}

.save-indicator:hover {
  background: #e4e7ed;
}

.zoom-indicator {
  min-width: 3.5rem;
  justify-content: center;
}

.word-count {
  cursor: default !important;
}

.zoom-controls {
  display: flex;
  flex-direction: row;
  align-items: center;
  height: 100%;
}

.word-count:hover {
  background: transparent !important;
}
</style>
