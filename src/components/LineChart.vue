<script lang="ts">
/*
import * as echarts from "echarts";

var chartDom = document.getElementById("main");
var myChart = echarts.init(chartDom);
var option;

option = {
  tooltip: {
    trigger: "axis",
    formatter: function (params) {
      return `${params[0].value[0]} is ${params[0].value[1]}`;
    },
  },
  xAxis: {
    type: "time",
    axisLabel: {
      formatter: "{MM}-{dd} {HH}:{mm}",
      showMinLabel: false,
      showMaxLabel: false,
      hideOverlap: true,
    },
  },
  yAxis: {
    type: "value",
    splitLine: {
      show: true,
    },
    axisLabel: {
      formatter: "{value}%",
    },
  },
  visualMap: [
    {
      type: "piecewise",
      show: false,
      dimension: 2,
      categories: [1, 0],
      inRange: {
        color: {
          1: "green",
          0: "red",
        },
      },
    },
  ],
  series: [
    {
      markArea: {
        data: [
          [
            {
              name: "放电",
              xAxis: "2025-1-3",
              itemStyle: {
                color: "rgba(255, 127, 127, 0.2)",
              },
            },
            {
              xAxis: "2025-1-4",
            },
          ],
          [
            {
              name: "满电",
              xAxis: "2025-1-5",
              itemStyle: {
                color: "rgba(0, 127, 127, 0.2)",
              },
            },
            {
              xAxis: "2025-1-6",
            },
          ],
        ],
      },
      data: [
        ["2025/1/1 12:05:01", 100, 0],
        ["2025/1/1 12:15:02", 100, 0],
        ["2025/1/1 12:30:03", 100, 0],
        ["2025/1/1 13:00:04", 100, 0],
        ["2025/1/1 14:00:05", 100, 1],
        ["2025/1/2 12:05:00", 80, 1],
        ["2025/1/3 14:00:00", 60],
        ["2025/1/4 15:00:00", 100],
        ["2025/1/5 12:00:00", 30],
        ["2025/1/6 12:00:00", 20],
        ["2025/1/7 12:00:00", 20, 1],
        ["2025/1/8 12:00:00", 20, 1],
        ["2025/1/8 13:00:00", 60, 0],
        ["2025/1/8 14:00:00", 100, 0],
        ["2025/1/10 12:00:00", 20],
        ["2025/1/11 12:00:00", 20],
      ],
      type: "line",
      smooth: true,
      sampling: "lttb",
      colorBy: "data",
    },
  ],
};

option && myChart.setOption(option);
*/
</script>
