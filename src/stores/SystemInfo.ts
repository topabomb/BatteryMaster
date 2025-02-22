import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { warn, debug, error } from "@tauri-apps/plugin-log";
export interface SystemInfo {
  cpu_name: string;
  cpu_vendor: string;
  support_power_set: boolean;
}
export const useStore = defineStore("SystemInfo", {
  state: (): SystemInfo => {
    return {
      cpu_name: "unknow",
      cpu_vendor: "unknow",
      support_power_set: false,
    };
  },
  getters: {},
  actions: {
    async load() {
      const val = (await invoke("get_system")) as SystemInfo;
      debug(`invoke:get_system-${JSON.stringify(val)}`);
      this.$patch(val);
      return val;
    },
  },
});
