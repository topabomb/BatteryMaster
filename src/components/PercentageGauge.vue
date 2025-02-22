<template>
  <v-chart
    :style="`min-height: ${props.height}px`"
    :option="option"
    autoresize
  />
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
import { title } from "process";
useEchart([
  CanvasRenderer,
  GaugeChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
]);
//provide(THEME_KEY, "dark");
const props = defineProps({
  name: String,
  value: Number,
  unit: String,
  height: { type: Number, default: 240 },
});
const value = computed(() => props.value);
const name = computed(() => props.name);
const tooltipFormater = computed(() => `{b} : {c} ${props.unit ?? "%"}`);
const detailFormater = computed(() => `{value} ${props.unit ?? "%"}`);
const option = ref({
  tooltip: {
    formatter: tooltipFormater,
  },
  title: {
    show: false,
  },
  series: [
    {
      type: "gauge",
      startAngle: 200,
      endAngle: -20,
      min: 0,
      max: 100,
      itemStyle: {
        color: "#26a69a",
      },
      title: { show: true, color: "#26a69a" },
      progress: {
        show: true,
        width: 20,
        itemStyle: {
          color: "#26a69a",
        },
      },
      axisLine: {
        lineStyle: {
          width: 20,
          color: [[1, "#c6cBc8"]],
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
        fontSize: 32,
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
