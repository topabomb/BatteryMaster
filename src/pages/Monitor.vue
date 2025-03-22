<template>
  <q-page class="q-pa-none">
    <q-tabs v-model="tab" dense class="text-grey" active-color="white">
      <q-tab name="full" label="满电" />
      <q-tab name="discharging" label="放电中" />
      <q-tab name="charging" label="充电中" />
      <q-tab name="empty" label="告急" />
    </q-tabs>
    <div class="row">
      <div class="col">
        <PercentageGauge
          name="CPU使用率"
          :value="Number((system_store.cpuload * 100).toFixed(1))"
        />
      </div>
      <div class="col">
        <PercentageGauge
          name="内存占用"
          :value="
            Number(
              (
                100 -
                (system_store.memfree / system_store.identifier.mem_total) * 100
              ).toFixed(1)
            )
          "
          unit="%"
        />
      </div>
      <div class="col">
        <PercentageGauge
          name="电量"
          :value="Number((battery_store.percentage! * 100).toFixed(1))"
          unit="%"
        />
      </div>
    </div>
    <q-tab-panels v-model="tab" animated class="bg-dark">
      <q-tab-panel name="full">
        <div class="row">
          <div class="col">
            <PercentageGauge
              name="电池寿命"
              :value="Number((battery_store.state_of_health! * 100).toFixed(1))"
            />
          </div>

          <div class="col">
            <PercentageGauge
              name="满充容量"
              :value="Number(battery_store.full_capacity!.toFixed(1))"
              unit="wh"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="设计容量"
              :value="Number(battery_store.design_capacity!.toFixed(1))"
              unit="wh"
            />
          </div>
        </div>
      </q-tab-panel>
      <q-tab-panel name="discharging">
        <div class="row">
          <div class="col">
            <PercentageGauge
              name="充放电功率"
              :value="Number(battery_store.energy_rate.toFixed(1))"
              unit="w"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="电池电压"
              :value="Number(battery_store.voltage.toFixed(1))"
              unit="v"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="当前容量"
              :value="Number(battery_store.capacity!.toFixed(1))"
              unit="wh"
            />
          </div>
        </div>
      </q-tab-panel>
      <q-tab-panel name="charging">
        <div class="row">
          <div class="col">
            <PercentageGauge
              name="充放电功率"
              :value="Number(battery_store.energy_rate.toFixed(1))"
              unit="w"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="电池电压"
              :value="Number(battery_store.voltage.toFixed(1))"
              unit="v"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="当前容量"
              :value="Number(battery_store.capacity!.toFixed(1))"
              unit="wh"
            />
          </div>
        </div>
      </q-tab-panel>
      <q-tab-panel name="empty"> </q-tab-panel>
    </q-tab-panels>
  </q-page>
</template>
<script setup lang="ts">
import { useQuasar } from "quasar";
import { computed, onMounted, ref, watch } from "vue";
import { useStore as useBatteryInfoStore } from "../stores/BatteryInfo";
import { useStore as useSystemInfo } from "../stores/SystemInfo";
import PercentageGauge from "../components/PercentageGauge.vue";
const battery_store = useBatteryInfoStore();
const system_store = useSystemInfo();
const tab = ref(battery_store.state.toLowerCase());
watch(
  () => battery_store.state,
  async (nVal) => {
    if (import.meta.env.MODE != "development") {
      tab.value = nVal.toLowerCase();
    }
  }
);
</script>
