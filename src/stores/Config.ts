import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
export interface Config {
  auto_start: boolean;
  tray_icon_update: number;
  service_update: number;
  record_battery_history: boolean;
}
export const useStore = defineStore("Config", {
  state: (): Config => {
    return {
      auto_start: false,
      tray_icon_update: 2,
      service_update: 1,
      record_battery_history: true,
    };
  },
  getters: {},
  actions: {
    async load() {
      const cfg = (await invoke("get_config")) as Config;
      this.$patch(cfg);
      return cfg;
    },
    async update(nVal: Config) {
      this.$patch(nVal);
      await invoke("set_config", {
        config: nVal,
      });
    },
  },
});
