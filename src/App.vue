<template>
  <q-layout view="hHh lpR fFf">
    <q-header class="text-white">
      <q-bar data-tauri-drag-region>
        <q-tabs
          v-model="tab"
          @update:model-value="tabChanged"
          dense
          stretch
          shrink
          active-color="white"
          class="text-grey"
          indicator-color="transparent"
        >
          <q-route-tab name="monitor" label="监控" to="/monitor" exact />
          <!--
          <q-route-tab name="history" label="历史" to="/history" exact />
          -->
          <q-route-tab name="cpupower" label="功耗" to="/cpupower" exact />
          <q-route-tab name="setting" label="设置" to="/setting" exact />
        </q-tabs>
        <q-space />
        <q-chip square :color="state_color" size="sm" class="text-white">
          {{ time_of_battery }}
        </q-chip>
        <q-btn
          size="sm"
          flat
          :icon="state_icon"
          :color="state_color"
          dense
          class="q-ml-none q-pl-none"
        >
          <q-chip square :color="state_color" size="sm" class="text-white">
            {{ (battery_store.percentage * 100).toFixed(1) }}%
          </q-chip>
        </q-btn>

        <q-separator vertical></q-separator>
        <q-btn dense flat icon="minimize" @click="minimize" />
        <q-btn dense flat icon="close" @click="close" />
      </q-bar>
    </q-header>

    <q-page-container>
      <router-view class="bg-dark" />
    </q-page-container>
  </q-layout>
</template>
<script setup lang="ts">
import { useQuasar } from "quasar";
import { invoke } from "@tauri-apps/api/core";
import { warn, debug, error } from "@tauri-apps/plugin-log";
import { formatDuration, intervalToDuration } from "date-fns";
import { onMounted, computed, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useStore as useBatteryInfo, BatteryInfo } from "./stores/BatteryInfo";
import { useRouter } from "vue-router";
import { useStore as useConfig, Config } from "./stores/Config";
import { useStore as usePower, ApuPower } from "./stores/ApuPower";
import { useStore as useSystem } from "./stores/SystemInfo";
import router from "./router";

const config_store = useConfig();
config_store.load().then();
const battery_store = useBatteryInfo();
battery_store.load().then();
const power_store = usePower();
power_store.load().then();
const sys_store = useSystem();
sys_store.load().then();
const state_color = computed(() => {
  switch (battery_store.state) {
    case "Full":
      return "grey";
    case "Charging":
      return "green";
    case "Discharging":
      return "orange";
    default:
      return "amber";
  }
});
const time_of_battery = computed(() => {
  switch (battery_store.state) {
    case "Full":
      return `${battery_store.capacity.toFixed(2)}wh`;
    case "Charging":
      return formatDuration(
        intervalToDuration({
          start: 0,
          end: battery_store.time_to_full_secs * 1000,
        }),
        { format: ["hours", "minutes"] }
      );
    case "Discharging":
      return formatDuration(
        intervalToDuration({
          start: 0,
          end: battery_store.time_to_empty_secs * 1000,
        }),
        { format: ["hours", "minutes"] }
      );
    default:
      return "unknow";
  }
});
const state_icon = computed(() => {
  switch (battery_store.state) {
    case "Full":
      return "battery_full";
    case "Charging":
      return battery_store.percentage >= 0.9
        ? "sym_o_battery_charging_90"
        : battery_store.percentage >= 0.8
          ? "sym_o_battery_charging_80"
          : battery_store.percentage >= 0.6
            ? "sym_o_battery_charging_60"
            : battery_store.percentage >= 0.5
              ? "sym_o_battery_charging_50"
              : battery_store.percentage >= 0.3
                ? "sym_o_battery_charging_30"
                : "sym_o_battery_charging_20";
    case "Discharging":
      return battery_store.percentage >= 0.9
        ? "battery_6_bar"
        : battery_store.percentage >= 0.8
          ? "battery_5_bar"
          : battery_store.percentage >= 0.6
            ? "battery_4_bar"
            : battery_store.percentage >= 0.5
              ? "battery_3_bar"
              : battery_store.percentage >= 0.3
                ? "battery_2_bar"
                : "battery_1_bar";
    default:
      return "battery_unknown";
  }
});
let tab = ref("");
onMounted(() => {
  router.push("/monitor");
});
const tabChanged = async (nval: string) => {
  if (!nval) return;
  let result: boolean = false;
  let param = {
    battery: true,
    system: false,
    power: false,
    config: false,
    log: false,
    history: false,
  };
  switch (nval) {
    case "monitor":
      result = (await invoke("set_event_channel", {
        setting: { ...param },
      })) as boolean;
      break;
    case "history":
      result = (await invoke("set_event_channel", {
        setting: { ...param, history: true },
      })) as boolean;
      break;
    case "cpupower":
      result = (await invoke("set_event_channel", {
        setting: { ...param, power: true },
      })) as boolean;
      break;
    case "setting":
      result = (await invoke("set_event_channel", {
        setting: { ...param, config: true },
      })) as boolean;
      break;
    default:
      break;
  }
  debug(`ui tab change to ${tab.value},call set_event_channel is ${result}`);
};

const minimize = () => {
  getCurrentWindow().minimize();
};
const close = () => {
  getCurrentWindow().close();
};
</script>
