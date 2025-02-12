<template>
  <v-chart style="height: 300px" :option="option" autoresize />
</template>
<script setup lang="ts">
import { use as useEchart } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { GaugeChart } from "echarts/charts";
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
} from "echarts/components";
import VChart, { THEME_KEY } from "vue-echarts";
import { provide, ref, computed } from "vue";
useEchart([
  CanvasRenderer,
  GaugeChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
]);
provide(THEME_KEY, "dark");
const props = defineProps({
  name: String,
  value: Number,
  unit: String,
});
const value = computed(() => props.value);
const name = computed(() => props.name);
const tooltipFormater = computed(() => `{b} : {c} ${props.unit ?? "%"}`);
const detailFormater = computed(() => `{value} ${props.unit ?? "%"}`);
const option = ref({
  tooltip: {
    formatter: tooltipFormater,
  },
  series: [
    {
      type: "gauge",
      startAngle: 200,
      endAngle: -20,
      min: 0,
      max: 100,
      splitNumber: 10,
      itemStyle: {
        color: "#FFAB91",
      },
      title: { show: true, color: "#FFAB91" },
      progress: {
        show: true,
        width: 40,
        itemStyle: {
          color: "#FFAB91",
        },
      },
      axisLine: {
        lineStyle: {
          width: 40,
        },
      },
      pointer: {
        show: false,
      },
      axisTick: {
        show: false,
      },
      splitLine: {
        show: false,
      },
      axisLabel: {
        show: false,
      },
      anchor: {
        show: false,
      },
      detail: {
        valueAnimation: true,
        width: "60%",
        offsetCenter: [0, "-10%"],
        fontSize: 36,
        fontWeight: "bolder",
        formatter: detailFormater,
        color: "inherit",
      },
      data: [
        {
          value: value,
          name: name,
        },
      ],
    },
  ],
});
</script>
