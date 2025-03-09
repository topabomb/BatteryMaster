import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { warn, debug, error } from "@tauri-apps/plugin-log";
export interface SystemInfo {
  identifier: {
    cpu_name: string;
    cpu_vendor: string;
    mem_total: number;
    hostname: string;
  };
  cpuload: number;
  memfree: number;
  screen_brightness: number;
  screen_instance: string;
  support_power_set: boolean;
}
let listenHandle: Promise<UnlistenFn>;
export const useStore = defineStore("SystemInfo", {
  state: (): SystemInfo => {
    return {
      identifier: {
        cpu_name: "Unknown",
        cpu_vendor: "Unknown",
        mem_total: 0,
        hostname: "Unknown",
      },
      support_power_set: false,
      cpuload: 0,
      memfree: 0,
      screen_brightness: 0,
      screen_instance: "Unknown",
    };
  },
  getters: {},
  actions: {
    async load() {
      const val = (await invoke("get_system")) as SystemInfo;
      debug(`invoke:get_system-${JSON.stringify(val)}`);
      this.$patch(val);

      listenHandle = listen<SystemInfo>("system_info_updated", async (e) => {
        await this.update(e.payload);
      });
    },
    async update(nVal: SystemInfo) {
      this.$patch(nVal);
    },
  },
});
