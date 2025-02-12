import { createMemoryHistory, createRouter } from "vue-router";
const routes = [
  {
    path: "/monitor",
    component: () => import("./pages/Monitor.vue"),
  },
  {
    path: "/history",
    component: () => import("./pages/History.vue"),
  },
  {
    path: "/cpupower",
    component: () => import("./pages/CpuPower.vue"),
  },
  {
    path: "/setting",
    component: () => import("./pages/Setting.vue"),
  },
];

const router = createRouter({
  history: createMemoryHistory(),
  routes,
});
export default router;
