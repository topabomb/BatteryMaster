import { defineStore } from "pinia";

export interface BatteryInfo {
  timestamp: number;
  state: string;
  percentage: number;
  energy_rate: number;
  cpu_load: number;
  cpu_model: string;
  capacity: number;
  design_capacity: number;
  state_of_health: number;
  voltage: number;
}
export const useStore = defineStore("BatteryInfo", {
  state: (): BatteryInfo => ({
    timestamp: 0,
    state: "",
    percentage: 0,
    energy_rate: 0,
    cpu_load: 0,
    cpu_model: "",
    capacity: 0,
    design_capacity: 0,
    state_of_health: 0,
    voltage: 0,
  }),
  getters: {},
  actions: {
    update(nVal: BatteryInfo) {
      this.$patch(nVal);
    },
  },
});
