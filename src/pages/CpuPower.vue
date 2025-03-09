<template>
  <q-page>
    <q-dialog v-model="warn_dialog">
      <q-card style="width: 320px" class="q-pa-xs">
        <q-list separator>
          <q-item dense>
            <q-item-section
              ><q-item-label class="text-h6">Warning</q-item-label>
              <q-item-label caption
                >您设置的功率限制部分没有生效，可能是因为计算机未支持部分设置，也有可能是我们不支持您的计算机；稍后谨慎进行功率限制，最好不要使用锁定功能；</q-item-label
              ></q-item-section
            >
          </q-item>
          <q-item dense
            ><q-item-section
              >下面是设置功率后实际查询到的值</q-item-section
            ></q-item
          >
          <q-item dense>
            <q-item-section> stapm_limit </q-item-section>
            <q-item-section side> {{ set_result.stapm_limit }}w</q-item-section>
          </q-item>

          <q-item dense>
            <q-item-section> slow_limit </q-item-section>
            <q-item-section side> {{ set_result.slow_limit }}w</q-item-section>
          </q-item>

          <q-item dense>
            <q-item-section> fast_limit </q-item-section>
            <q-item-section side> {{ set_result.fast_limit }}w</q-item-section>
          </q-item>
        </q-list>
        <q-card-actions align="right">
          <q-btn flat @click="warn_dialog = false">关闭</q-btn>
        </q-card-actions>
      </q-card>
    </q-dialog>
    <q-banner
      dense
      inline-actions
      class="text-white bg-red"
      v-show="!power_store.isAdmin"
    >
      需要以管理员模式启动
      <template v-slot:action>
        <q-btn
          size="sm"
          flat
          color="white"
          label="管理员模式"
          icon="restart_alt"
          @click="exec_elevate_self"
        />
      </template>
    </q-banner>
    <q-banner
      dense
      class="text-white bg-warning"
      v-show="sys_store.support_power_set && power_store.isAdmin"
      >该功能仅在支持的amd cpu上可用，也可能破坏您的计算机硬件,请谨慎使用；{{
    }}</q-banner>

    <q-form @submit="onSubmit" :loading="loading">
      <q-list separator>
        <q-item>
          <q-item-section>
            <q-item-label class="text-white">{{
              sys_store.identifier.cpu_vendor
            }}</q-item-label>
            <q-item-label caption class="text-grey"
              >{{ sys_store.identifier.cpu_name }}-{{
                power_store.identifier.cpu_family
              }}</q-item-label
            >
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.stapm_limit"
              show-value
              size="100px"
              color="light-green-6"
              track-color="light-green-2"
              class="text-light-green"
            ></q-knob>
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.stapm_limit"
              :min="5"
              :max="120"
              label
              label-always
              :disable="setting_disabled"
            /><q-item-label class="text-light-green text-h6"
              >长时功耗(w)</q-item-label
            >
            <q-item-label class="text-grey" caption
              >在没有触碰温度墙或其他因素，CPU可以长时间维持的最大功率限制；</q-item-label
            >
          </q-item-section>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.stamp_value"
              show-value
              size="100px"
              color="amber-6"
              track-color="amber-2"
              class="text-amber"
            ></q-knob>
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.slow_limit"
              show-value
              size="100px"
              color="light-green-6"
              track-color="light-green-2"
              class="text-light-green"
            ></q-knob>
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.slow_limit"
              :inner-min="form_value.stapm_limit"
              :min="5"
              :max="120"
              label
              label-always
              :disable="setting_disabled"
            /><q-item-label class="text-light-green text-h6"
              >短时功耗(w)</q-item-label
            >
            <q-item-label class="text-grey" caption
              >CPU可以短时间维持的最大功率限制</q-item-label
            >
          </q-item-section>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.slow_value"
              show-value
              size="100px"
              color="amber-6"
              track-color="amber-2"
              class="text-amber"
            ></q-knob>
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.fast_limit"
              show-value
              size="100px"
              color="light-green-6"
              track-color="light-green-2"
              class="text-light-green"
            ></q-knob>
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.fast_limit"
              :inner-min="form_value.stapm_limit"
              :min="5"
              :max="120"
              label
              label-always
              :disable="setting_disabled"
            /><q-item-label class="text-light-green text-h6"
              >瞬时功耗(w)</q-item-label
            >
            <q-item-label class="text-grey" caption
              >CPU在瞬间能达到的最高功率限制</q-item-label
            >
          </q-item-section>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.fast_value"
              show-value
              size="100px"
              color="amber-6"
              track-color="amber-2"
              class="text-amber"
            ></q-knob>
          </q-item-section>
        </q-item>
        <q-item tag="label" v-ripple>
          <q-item-section>
            <q-item-label class="text-white">自动锁定</q-item-label>
            <q-item-label caption class="text-grey-5"
              >可能有其他进程重新设置功率限制，锁定后将每隔10秒恢复到设置的值。</q-item-label
            >
          </q-item-section>
          <q-item-section side top>
            <q-toggle
              v-model="form_value.auto_lock"
              @update:model-value="auto_lock_change"
              :disable="!power_store.isAdmin || !form_value.modifyed"
            />
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section>
            <q-btn
              color="grey"
              label="恢复"
              icon="restart_alt"
              :loading="loading"
              @click="onReset"
              :disable="
                btn_disabled || !form_value.modifyed || form_value.auto_lock
              "
            >
              <q-tooltip class="q-pa-none">
                <q-list bordered separator v-if="!!power_store.init_value">
                  <q-item dense>
                    <q-item-section>
                      <q-item-label
                        >即将恢复到如下设置</q-item-label
                      ></q-item-section
                    >
                  </q-item>
                  <q-item dense>
                    <q-item-section avatar>stapm_limit</q-item-section>
                    <q-item-section
                      ><q-item-label>{{
                        power_store.init_value!.stapm_limit
                      }}</q-item-label></q-item-section
                    >
                  </q-item>
                  <q-item dense>
                    <q-item-section avatar>slow_limit</q-item-section>
                    <q-item-section
                      ><q-item-label>{{
                        power_store.init_value!.slow_limit
                      }}</q-item-label></q-item-section
                    >
                  </q-item>
                  <q-item dense>
                    <q-item-section avatar>fast_limit</q-item-section>
                    <q-item-section
                      ><q-item-label>{{
                        power_store.init_value!.fast_limit
                      }}</q-item-label></q-item-section
                    >
                  </q-item>
                </q-list></q-tooltip
              >
            </q-btn>
          </q-item-section>
          <q-item-section>
            <q-btn
              color="primary"
              label="设置"
              icon="save"
              :loading="loading"
              :disable="btn_disabled || !can_submit || form_value.auto_lock"
              @click="onSubmit"
            ></q-btn>
          </q-item-section>
        </q-item>
      </q-list>
    </q-form>
  </q-page>
</template>
<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useStore as usePower, LimitSet } from "../stores/ApuPower";
import { useStore as userBattery } from "../stores/BatteryInfo.ts";
import { useStore as useSystem } from "../stores/SystemInfo";
import { invoke } from "@tauri-apps/api/core";

import { useQuasar } from "quasar";
const $q = useQuasar();
const power_store = usePower();
power_store.refresh().then();
const sys_store = useSystem();
const loading = ref(false);
const warn_dialog = ref(false);
const enable_autolock = ref(false);

const setting_disabled = computed(() => {
  return (
    form_value.value.auto_lock ||
    !power_store.isAdmin ||
    !sys_store.support_power_set
  );
});
const btn_disabled = computed(() => {
  return !power_store.isAdmin || !sys_store.support_power_set;
});
const can_submit = computed(() => {
  return (
    form_value.value.fast_limit != power_store.fast_limit ||
    form_value.value.slow_limit != power_store.slow_limit ||
    form_value.value.stapm_limit != power_store.stapm_limit
  );
});
const set_result = ref({} as LimitSet);
const set_limit = async (val: LimitSet) => {
  loading.value = true;

  const result = await power_store.set_limit({
    ...val,
  });
  $q.notify(`set apu limit ${result?.[0]}`);
  if (result && !result[0]) {
    set_result.value.fast_limit = result![1].fast_limit;
    set_result.value.slow_limit = result![1].slow_limit;
    set_result.value.stapm_limit = result![1].stapm_limit;
    warn_dialog.value = true;
  } else if (result && result[0]) {
    enable_autolock.value = true;
  }
  loading.value = false;
};
const onSubmit = async () => {
  if (form_value.value.slow_limit < form_value.value.stapm_limit)
    form_value.value.slow_limit = form_value.value.stapm_limit;
  if (form_value.value.fast_limit < form_value.value.stapm_limit)
    form_value.value.fast_limit = form_value.value.stapm_limit;
  await set_limit(form_value.value);
  form_value.value.modifyed = true;
};
const onReset = async () => {
  if (power_store.init_value) {
    await set_limit(power_store.init_value);
    form_value.value.modifyed = false;
  }
};
const form_value = ref(power_store.form_value);
const exec_elevate_self = async () => {
  await invoke("exec_elevate_self");
};
const auto_lock_change = async () => {
  await power_store.set_limit_lock(form_value.value.auto_lock, {
    ...form_value.value,
  } as LimitSet);
};
</script>
