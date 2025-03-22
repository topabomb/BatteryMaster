import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { warn, debug, error } from "@tauri-apps/plugin-log";
export interface HistoryInfo {
  prev_timestamp?: number;
  timestamp_diff?: number;
  prev_state_of_health?: number;
  state_of_health_diff?: number;
  prev_percentage?: number;
  percentage_diff?: number;
  prev_capacity?: number;
  capacity_diff?: number;

  timestamp: number;
  end_at?: number;
  state: string;
  prev?: string;
  capacity: number;
  full_capacity: number;
  design_capacity: number;
  percentage: number;
  state_of_health: number;
  energy_rate: number;
  voltage: number;
  cpu_load: number;
}
let listenHandle: Promise<UnlistenFn>;
export const useStore = defineStore("HistoryInfo", {
  state: () => {
    return {};
  },
  getters: {},
  actions: {
    load: async () => {
      listenHandle = listen<boolean>("history_info_updated", async () => {});
    },
    history_page: async (cursor?: number, size?: number) => {
      let res = await invoke<HistoryInfo[]>("get_battery_history_page", {
        cursor,
        size: size ?? 10,
      });
      return res;
    },
    history: async (id: number) => {
      return await invoke<HistoryInfo>("get_battery_history", { id });
    },
  },
});
