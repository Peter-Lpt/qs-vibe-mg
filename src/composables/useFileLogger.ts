import { invoke } from "@tauri-apps/api/core";

export function useFileLogger() {
  async function log(level: "info" | "warn" | "error" | "debug", message: string) {
    try {
      await invoke("log_message", { level, message });
    } catch {
      // 静默失败
    }
  }

  return {
    info: (msg: string) => log("info", msg),
    warn: (msg: string) => log("warn", msg),
    error: (msg: string) => log("error", msg),
    debug: (msg: string) => log("debug", msg),
  };
}
