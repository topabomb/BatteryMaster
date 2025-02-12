<template>
  <q-page class="q-pa-xs bg-dark">
    <div class="row">
      <div class="col">
        <PercentageGauge
          name="CPU使用率"
          :value="Number((battery_store.cpu_load * 100).toFixed(1))"
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
          name="充放电功率"
          :value="Number(battery_store.energy_rate.toFixed(1))"
          unit="w"
        />
      </div>
    </div>
    <div class="row">
      <div class="col">
        <PercentageGauge
          name="电量"
          :value="Number((battery_store.percentage! * 100).toFixed(1))"
          unit="%"
        />
      </div>
      <div class="col">
        <PercentageGauge
          name="电池寿命"
          :value="Number((battery_store.state_of_health! * 100).toFixed(1))"
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
  </q-page>
</template>
<script setup lang="ts">
import { useQuasar } from "quasar";
import { onMounted, ref } from "vue";
import {
  useStore as useBatteryInfoStore,
  BatteryInfo,
} from "../stores/BatteryInfo";
import PercentageGauge from "../components/PercentageGauge.vue";
const battery_store = useBatteryInfoStore();
</script>
