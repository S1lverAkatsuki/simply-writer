<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";

const scrollWrapperRef = ref<HTMLElement | null>(null);
const pageRef = ref<HTMLTextAreaElement | null>(null);
const text = ref("");
const title = ref("aaa");

const wordCount = computed(() => text.value.replace(/\s/g, "").length);

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

onMounted(async () => {
  document.title = title.value;
  await nextTick();
  syncPageHeight();
});

const zoomLevel = ref<number>(1);

const handleCtrlWheel = (e: WheelEvent) => {
  if (!e.ctrlKey) return;
  e.stopPropagation();
  const delta = e.deltaY > 0 ? -0.25 : 0.25;
  zoomLevel.value = Number(
    Math.max(0.5, Math.min(3, zoomLevel.value + delta)).toFixed(2),
  );
};

const resetZoomLevel = () => {
  zoomLevel.value = 1;
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === "Tab") {
    e.preventDefault();
    // 看到下面划线的方法没
    // 想要不用？那就写一长串 Range 和 Selection API 的东西吧，最后还不支持原生的撤销栈
    document.execCommand("insertText", false, "\t");
  }
};

const handleInput = async () => {
  await nextTick();
  syncPageHeight();
};
</script>

<template>
  <main class="viewport" @wheel.ctrl.prevent="handleCtrlWheel">
    <div class="scroll-wrapper" ref="scrollWrapperRef">
      <div
        class="zoom-container"
        :style="{
          width: `calc(210mm * ${zoomLevel})`,
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
      <span class="zoom-indicator" @click="resetZoomLevel"
        >{{ Math.round(zoomLevel * 100) }}%</span
      >
      <span>{{ wordCount }} 个字</span>
    </div>
  </main>
</template>

<style scoped>
.status-bar {
  z-index: 1;
  width: 100vw;
  height: 1em;
  bottom: 0;
  line-height: 1em;
  border-top: 1px solid white;
  background-color: #f5f5f5;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.status-bar > span {
  display: inline-block;
  height: 100%;
  font-size: 0.5em;
  user-select: none;
  vertical-align: middle;
  padding: 0 2em;
  color: #606266;
}

.status-bar > span:hover {
  background: #dcdfe6;
  cursor: pointer;
}

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
  touch-action: none;
}

.scroll-wrapper {
  width: 100%;
  height: 100%;

  flex: 1;

  overflow-y: auto;
  overflow-x: auto;
  display: block;

  padding-top: 3em;
  padding-bottom: 3em;
}

.zoom-container {
  width: 100%;
  height: 100%;

  margin: 0 auto;
  padding: 10em 0;
  position: relative;
  display: flex;
  justify-content: center;
  min-height: 100%;
  box-sizing: border-box;
}

.page {
  transform-origin: top center;

  position: absolute;
  top: 0;

  tab-size: 4;
  width: 210mm;
  height: auto;
  min-height: 297mm;

  display: block;
  margin-bottom: 10em;
  margin-left: auto;
  margin-right: auto;

  padding: 4em;
  background-color: white;
  box-shadow: 0 0.2em 1.5em rgba(0, 0, 0, 0.1);
  outline: none;
  border: 1px solid #dcdfe6;

  word-break: break-all;
  overflow-wrap: anywhere;
  white-space: break-spaces;
  text-rendering: optimizeLegibility;

  font-size: 15px;
  line-height: 1.6;
  cursor: text;

  resize: none;
  overflow-y: hidden;
}
</style>
