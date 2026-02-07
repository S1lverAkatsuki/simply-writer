<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";

const scrollWrapperRef = ref<HTMLElement | null>(null);
const pageRef = ref<HTMLTextAreaElement | null>(null);
const fileSelectorRef = ref<HTMLInputElement | null>(null);

const text = ref<string | null>(null);
const title = ref<string | null>(null);

const downloaderRef = ref<HTMLAnchorElement | null>(null);
const downloadUrl = ref<string | null>(null);

const hasInit = ref<boolean>(false);

const DEFAULT_TITLE: string = "新文档";

const purifyFilename = (raw: string) => raw.replace(/[\/:*?"<>|]/g, "_");

const requestTitle = (
  message: string,
  options: { initial?: string; fallback?: string } = {},
): string | null => {
  const placeholder = options.initial ?? title.value ?? DEFAULT_TITLE;
  const input = prompt(message, placeholder);
  if (input === null) return null;
  const trimmed = input.trim();
  const finalValue =
    trimmed.length > 0
      ? trimmed
      : (options.fallback ?? placeholder ?? DEFAULT_TITLE);
  return purifyFilename(finalValue);
};

const handleChangedTitle = () => {
  const value = requestTitle("输入新的标题", {
    initial: title.value ?? DEFAULT_TITLE,
    fallback: DEFAULT_TITLE,
  });
  if (value !== null) {
    title.value = value;
  }
};

const initWithDefaultFile = () => {
  const result = requestTitle("输入标题（留空则使用默认）", {
    initial: DEFAULT_TITLE,
    fallback: DEFAULT_TITLE,
  });
  if (result === null) return;

  hasInit.value = true;
  text.value = "";
  title.value = result;
};

const confirmLeave = (event: BeforeUnloadEvent) => {
  if (!hasInit.value) return;
  event.preventDefault();
};

onMounted(() => {
  window.addEventListener("beforeunload", confirmLeave);
});

onUnmounted(() => {
  window.removeEventListener("beforeunload", confirmLeave);
});

const downloadFilename = computed(() => {
  const trimmed = title.value?.trim();
  const safe = trimmed && trimmed.length > 0 ? trimmed : DEFAULT_TITLE;
  return `${safe}.txt`;
});

const displayTitle = computed(() => title.value?.trim() || DEFAULT_TITLE);

const handleSaveFile = async () => {
  if (!hasInit.value) return;

  if (downloadUrl.value) {
    URL.revokeObjectURL(downloadUrl.value);
    downloadUrl.value = null;
  }

  const payload = text.value ?? "";
  const blob = new Blob([payload], { type: "text/plain;charset=utf-8" });
  downloadUrl.value = URL.createObjectURL(blob);

  // 先确保隐藏的 a 标签信息以及写入进去，再点击才能更新
  await nextTick();

  if (downloaderRef.value) {
    downloaderRef.value.download = downloadFilename.value;
    downloaderRef.value.click();
  }

  // 先去处理下载，再在下一个事件循环里释放 ObjectURL
  await nextTick();

  if (downloadUrl.value) {
    URL.revokeObjectURL(downloadUrl.value);
    downloadUrl.value = null;
  }
};

const triggerFileSelect = () => fileSelectorRef.value?.click();
const initWithCustomFile = () => {
  const selector = fileSelectorRef.value;
  const file = selector?.files?.[0];
  if (!selector || !file) return;

  const reader = new FileReader();

  reader.onload = async () => {
    text.value = typeof reader.result === "string" ? reader.result : "";
    const rawName = file.name.replace(/\.[^/.]+$/, "");
    const normalized = rawName.trim() || DEFAULT_TITLE;
    title.value = purifyFilename(normalized);
    hasInit.value = true;
    selector.value = ""; // 我也搞不懂为什么 ref 是字符串，好像是说拿到的只是个伪路径
    await nextTick();
    syncPageHeight();
  };

  reader.readAsText(file, "utf-8");
};

const wordCount = computed(() => {
  if (!text.value) return 0;
  // 统计的是非空字符的数量
  return text.value.replace(/\s/g, "").length;
});

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

watch(title, async () => {
  document.title = displayTitle.value;
  await nextTick();
  syncPageHeight();
});

const zoomLevel = ref<number>(1);

const changeZoomLevel = (delta: number) =>
  (zoomLevel.value = Number(
    Math.max(0.5, Math.min(3, zoomLevel.value + delta)).toFixed(2),
  ));
const handleCtrlWheel = (e: WheelEvent) => {
  if (!e.ctrlKey) return;
  e.stopPropagation();
  const delta = e.deltaY > 0 ? -0.25 : 0.25;
  changeZoomLevel(delta);
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

  if ((e.key === "s" || e.key === "S") && (e.ctrlKey || e.metaKey)) {
    e.preventDefault();
    handleSaveFile();
  }
};

const handleInput = async () => {
  await nextTick();
  syncPageHeight();
};
</script>

<template>
  <main class="viewport" @wheel.ctrl.prevent="handleCtrlWheel">
    <div v-if="!hasInit" class="init-prompt">
      <!-- 看不到这个 -->
      <input
        type="file"
        ref="fileSelectorRef"
        class="u-cannt-see-me"
        accept=".txt"
        @change="initWithCustomFile"
      />
      <button type="button" @click="triggerFileSelect">选择文件</button>
      <span style="text-align: center">OR</span>
      <button @click="initWithDefaultFile">直接进入</button>
    </div>
    <template v-else>
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
        <span @click="handleChangedTitle">更改标题</span>
        <span @click="handleSaveFile">保存</span>
        <div class="status-right">
          <div class="zoom-controls">
            <span @click="changeZoomLevel(-0.25)">-</span>
            <span class="zoom-indicator" @click="resetZoomLevel">
              {{ Math.round(zoomLevel * 100) }}%
            </span>
            <span @click="changeZoomLevel(0.25)">+</span>
          </div>
          <span>{{ wordCount }} 个字</span>
        </div>
      </div>
    </template>
    <a
      v-if="downloadUrl"
      ref="downloaderRef"
      class="u-cannt-see-me"
      :href="downloadUrl"
      :download="downloadFilename"
    />
  </main>
</template>

<style scoped>
.init-prompt {
  background-color: white;
  border: 1px solid #e4e7ed;
  width: 24em;
  height: 20em;
  margin: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1em;
  border-radius: 0.5em;
  box-shadow: 0 0.6em 2em rgba(0, 0, 0, 0.08);
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
  font-size: 0.9em;
}

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
  justify-content: space-between;
}

.status-bar span {
  display: inline-block;
  height: 100%;
  font-size: 0.5em;
  user-select: none;
  vertical-align: middle;
  padding: 0 1em;
  color: #606266;
}

.status-bar span:hover {
  background: #dcdfe6;
  cursor: pointer;
}

.status-right {
  display: flex;
  align-items: center;
  gap: 1em;
}

.zoom-controls {
  display: flex;
  align-items: center;
}

.zoom-button:hover {
  background: #f5f7fa;
}

.zoom-indicator {
  width: 6em;
  text-align: center;
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

  padding: 3em;
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
