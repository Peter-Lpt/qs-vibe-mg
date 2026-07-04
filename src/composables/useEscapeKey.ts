import { onMounted, onUnmounted } from "vue";

export function useEscapeKey(handler: () => void) {
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      handler();
    }
  }

  onMounted(() => document.addEventListener("keydown", handleKeydown));
  onUnmounted(() => document.removeEventListener("keydown", handleKeydown));
}
