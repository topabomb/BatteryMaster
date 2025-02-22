import { defineStore } from "pinia";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
export interface BatteryInfo {
  timestamp: number;
  serial_number: "";
  state: string;
  percentage: number;
  energy_rate: number;
  cpu_load: number;
  cpu_model: string;
  capacity: number;
  design_capacity: number;
  state_of_health: number;
  voltage: number;
  time_to_empty_secs: number;
  time_to_full_secs: number;
}
let listenHandle: Promise<UnlistenFn>;
export const useStore = defineStore("BatteryInfo", {
  state: (): BatteryInfo => ({
    timestamp: 0,
    serial_number: "",
    state: "",
    percentage: 0,
    energy_rate: 0,
    cpu_load: 0,
    cpu_model: "",
    capacity: 0,
    design_capacity: 0,
    state_of_health: 0,
    voltage: 0,
    time_to_empty_secs: 0,
    time_to_full_secs: 0,
  }),
  getters: {},
  actions: {
    async load() {
      listenHandle = listen<BatteryInfo>("battery_info_updated", async (e) => {
        await this.update(e.payload);
      });
    },
    async update(nVal: BatteryInfo) {
      this.$patch(nVal);
    },
  },
});
