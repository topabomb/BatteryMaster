import { defineStore } from "pinia";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
export interface BatteryInfo {
  identifier: {
    serial_number: string;
    vendor: string;
    model: string;
  };
  state_changed: boolean;
  timestamp: number;
  state: string;
  percentage: number;
  energy_rate: number;
  capacity: number;
  full_capacity: number;
  design_capacity: number;
  state_of_health: number;
  voltage: number;
  time_to_empty_secs: number;
  time_to_full_secs: number;
}
let listenHandle: Promise<UnlistenFn>;
export const useStore = defineStore("BatteryInfo", {
  state: (): BatteryInfo => ({
    identifier: {
      serial_number: "",
      vendor: "",
      model: "",
    },
    state_changed: false,
    timestamp: 0,
    state: "",
    percentage: 0,
    energy_rate: 0,
    capacity: 0,
    full_capacity: 0,
    design_capacity: 0,
    state_of_health: 0,
    voltage: 0,
    time_to_empty_secs: 0,
    time_to_full_secs: 0,
  }),
  getters: {},
  actions: {
    async load() {
      const val = (await invoke("get_battery")) as BatteryInfo;
      this.$patch(val);
      listenHandle = listen<BatteryInfo>("battery_info_updated", async (e) => {
        await this.update(e.payload);
      });
    },
    async update(nVal: BatteryInfo) {
      this.$patch(nVal);
    },
  },
});
