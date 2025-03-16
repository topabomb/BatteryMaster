<template>
  <q-page>
    <q-banner
      dense
      inline-actions
      class="text-white bg-warning"
      v-show="!config_store.record_battery_history"
    >
      在设置中打开记录历史功能方可跟踪电池健康度；
      <template v-slot:action>
        <q-btn
          size="sm"
          flat
          color="white"
          label="去设置"
          icon="settings"
          to="/setting"
        />
      </template>
    </q-banner>

    <div class="q-pa-xs">
      <!--<q-scroll-area style="height: 238px">-->
      <q-infinite-scroll :offset="250" @load="onLoadMore" ref="infiniteScroll">
        <template v-slot:loading>
          <div class="row justify-center q-my-sm">
            <q-spinner-dots color="primary" size="32px" />
          </div>
        </template>
        <q-list dark separator dense>
          <q-item v-for="row in list" :key="row.timestamp" clickable v-ripple>
            <q-item-section avatar
              ><q-icon
                :name="state_icon(row.state)"
                :class="state_color(row.state)"
            /></q-item-section>
            <q-item-section style="max-width: 200px"
              ><q-item-label caption
                >{{ state_text(row.state) }}
                {{
                  row.end_at
                    ? formatDuration(
                        intervalToDuration({
                          start: row.timestamp * 1000,
                          end: row.end_at * 1000,
                        }),
                        {
                          format:
                            row.end_at - row.timestamp > 600
                              ? ["days", "hours", "minutes"]
                              : ["minutes", "seconds"],
                        }
                      )
                    : ""
                }}</q-item-label
              >
              <q-item-label class="text-h6"
                >{{
                  row.state != "full"
                    ? `${row.prev_percentage != null ? (row.prev_percentage * 100).toFixed(0) : "nan"}-`
                    : ""
                }}{{ (row.percentage * 100).toFixed(0) }}%
              </q-item-label>
              <q-item-label caption
                >CPU占用{{ (row.cpu_load * 100).toFixed(1) }}%</q-item-label
              >
            </q-item-section>
            <q-item-section
              ><q-item-label caption
                >开始于
                {{ format(row.timestamp * 1000, "MM-dd hh:mm") }}</q-item-label
              ><q-item-label class="text-h6"
                ><span :class="state_color(row.state)">
                  {{
                    row.state != "full"
                      ? row.percentage_diff != null
                        ? `${(row.percentage_diff * 100).toFixed(2)}%`
                        : "nan"
                      : `${row.capacity.toFixed(2)}wh`
                  }}</span
                ></q-item-label
              ><q-item-label caption
                >健康度{{
                  (row.state_of_health * 100).toFixed(1)
                }}%</q-item-label
              >
            </q-item-section>
            <q-item-section side>
              <q-item-label caption :class="state_color(row.state)">
                {{
                  row.state != "full"
                    ? row.capacity_diff != null
                      ? `${row.capacity_diff.toFixed(1)}wh`
                      : `nan`
                    : ``
                }}</q-item-label
              >
              <q-item-label class="text-h6">
                <span :class="state_color(row.state)"
                  >{{ row.energy_rate.toFixed(1) }}w</span
                ></q-item-label
              >
              <q-item-label caption
                >{{
                  row.state_of_health_diff
                    ? (row.state_of_health_diff * 100).toFixed(1)
                    : 0
                }}%</q-item-label
              ></q-item-section
            >
          </q-item>
        </q-list>
      </q-infinite-scroll>
      <!--</q-scroll-area>
      <q-separator class="q-my-xs bg-primary"></q-separator>
      -->
    </div>
  </q-page>
</template>
<script setup lang="ts">
import {
  formatDistance,
  formatDuration,
  intervalToDuration,
  format,
  intlFormat,
} from "date-fns";

import { useQuasar } from "quasar";
import { useStore as useBatteryInfo, BatteryInfo } from "../stores/BatteryInfo";
import { useStore as useConfig, Config } from "../stores/Config";
import { useStore as usePower, PowerInfo } from "../stores/ApuPower";
import { useStore as useSystem } from "../stores/SystemInfo";
import { useStore as useHistory, HistoryInfo } from "../stores/HistoryInfo";
import { ref } from "vue";
import { listen } from "@tauri-apps/api/event";
const $q = useQuasar();
const config_store = useConfig();
const battery_store = useBatteryInfo();
const history_sotre = useHistory();
const state_text = (state: string) => {
  if (state == "full") return "满电";
  else if (state == "charging") return "充电";
  else if (state == "discharging") return "放电";
};
const state_color = (state: string) => {
  if (state == "full") return "text-grey";
  else if (state == "charging") return "text-green";
  else if (state == "discharging") return "text-red";
};
const state_icon = (state: string) => {
  if (state == "full") return "battery_full";
  else if (state == "charging") return "battery_saver";
  else if (state == "discharging") return "battery_charging_full";
};

const cursor = ref(Math.round(Date.now() / 1000));
const list = ref([] as HistoryInfo[]);
const infiniteScroll = ref(null);
listen<boolean>("history_info_updated", async (e) => {
  if (e) {
    const new_rows = await history_sotre.history_page(
      Math.round(Date.now() / 1000) + 1,
      1
    );
    console.log(new_rows);
    if (new_rows && new_rows.length > 0) list.value.unshift(...new_rows);
  }
}).then();
const onLoadMore = (_: number, done?: (stop?: boolean) => void) => {
  history_sotre
    .history_page(cursor.value, 5)
    .then((rows) => {
      console.log(rows);
      let over = false;
      if (rows && rows.length > 0) {
        list.value = list.value.concat(rows);
        const last = rows[rows.length - 1].timestamp;
        last != cursor.value && (cursor.value = last);
        console.log(cursor.value);
      } else over = true;
      done && done(over);
    })
    .catch((e) => {
      console.error(e);
      done && done(true);
    });
};
</script>
