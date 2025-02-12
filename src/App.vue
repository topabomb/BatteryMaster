<template>
  <q-layout view="hHh lpR fFf">
    <q-header class="text-white">
      <q-bar data-tauri-drag-region>
        <q-tabs
          dense
          stretch
          shrink
          active-color="white"
          class="text-grey"
          indicator-color="transparent"
        >
          <q-route-tab name="monitor" label="监控" to="/monitor" exact />
          <q-route-tab name="history" label="历史" to="/history" exact />
          <q-route-tab name="cpupower" label="功耗" to="/cpupower" exact />
          <q-route-tab name="setting" label="设置" to="/setting" exact />
        </q-tabs>

        <q-space />
        <q-btn dense flat :icon="state_icon" :color="state_color" to="/monitor">
          <q-badge :color="state_color" transparent
            >{{ (battery_store.percentage * 100).toFixed(1) }}%</q-badge
          >
        </q-btn>
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
import { onMounted, computed, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  useStore as useBatteryInfoStore,
  BatteryInfo,
} from "./stores/BatteryInfo";
import { useRouter, useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { useStore as useConfig, Config } from "./stores/Config";
const config_store = useConfig();
config_store.load().then();
const battery_store = useBatteryInfoStore();
listen<BatteryInfo>("battery_info_updated", (e) => {
  battery_store.update(e.payload);
});
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
const router = useRouter();
onMounted(() => {
  router.push("/monitor");
});
const minimize = () => {
  getCurrentWindow().minimize();
};
const close = () => {
  getCurrentWindow().close();
};
</script>
