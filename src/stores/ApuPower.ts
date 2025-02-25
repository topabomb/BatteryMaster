import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { warn, debug, error } from "@tauri-apps/plugin-log";
export interface LimitSet {
  stapm_limit: number;
  fast_limit: number;
  slow_limit: number;
}
export interface FormValue extends LimitSet {
  auto_lock: boolean;
  modifyed: boolean;
}
export interface PowerInfo extends LimitSet {
  table: number;
  cpu_family: number;
  stamp_value: number;
  fast_value: number;
  slow_value: number;
}
export interface ApuPower extends PowerInfo {
  isAdmin: boolean;
  form_value: FormValue;
  init_value: LimitSet | undefined;
}
let listenHandle: Promise<UnlistenFn>;
export const useStore = defineStore("ApuPower", {
  state: (): ApuPower => {
    return {
      form_value: {
        auto_lock: false,
        modifyed: false,
        stapm_limit: 12,
        slow_limit: 15,
        fast_limit: 15,
      },
      init_value: undefined,
      isAdmin: false,
      table: 0,
      cpu_family: 0,
      stapm_limit: 0,
      stamp_value: 0,
      fast_limit: 0,
      fast_value: 0,
      slow_limit: 0,
      slow_value: 0,
    };
  },
  getters: {},
  actions: {
    async load() {
      listenHandle = listen<PowerInfo>("power_info_updated", async (e) => {
        if (!this.init_value) {
          this.init_value = {
            fast_limit: e.payload.fast_limit,
            slow_limit: e.payload.slow_limit,
            stapm_limit: e.payload.stapm_limit,
          };
        }
        await this.update(e.payload);
      });
      this.isAdmin = (await invoke("get_isadmin")) as boolean;
    },
    async refresh() {
      if (this.isAdmin) {
        const info = (await invoke("get_powerinfo")) as PowerInfo;
        this.$patch(info);
      }
    },
    async update(nVal: PowerInfo) {
      this.$patch(nVal);
    },
    async set_limit(val: LimitSet) {
      if (this.isAdmin) {
        let result = (await invoke("set_power_limit", { limit: val })) as [
          boolean,
          PowerInfo,
        ];
        debug(`invoke:set_power_limit-${JSON.stringify(result)}`);
        return result;
      }
    },
    async set_limit_lock(enable: boolean, val: LimitSet) {
      if (this.isAdmin) {
        let result = (await invoke("set_power_limit_lock", {
          lock: enable,
          limit: val,
        })) as boolean;
        debug(
          `invoke:set_power_limit_lock(${enable})-${JSON.stringify(result)}`
        );
        return result;
      }
    },
  },
});
