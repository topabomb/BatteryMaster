<template>
  <q-page>
    <q-form @submit="onSubmit" class="text-grey-2" :loading="loading">
      <q-list bordered padding class="text-grey-3">
        <q-item tag="label" v-ripple>
          <q-item-section>
            <q-item-label>自动启动</q-item-label>
            <q-item-label caption class="text-grey-5"
              >随操作系统一同启动</q-item-label
            >
          </q-item-section>
          <q-item-section side top>
            <q-toggle v-model="form_value.auto_start" />
          </q-item-section>
        </q-item>
        <q-separator spaced />
        <!--
        <q-item tag="label" v-ripple>
          <q-item-section>
            <q-item-label>记录历史</q-item-label>
            <q-item-label caption class="text-grey-5"
              >记录电池消耗历史</q-item-label
            >
          </q-item-section>
          <q-item-section side top>
            <q-toggle v-model="form_value.record_battery_history" />
          </q-item-section>
        </q-item>
        <q-separator spaced />
        -->
        <q-item-label header class="text-grey-3">电池信息更新间隔</q-item-label>
        <q-item>
          <q-item-section side>
            <q-icon color="primary" name="schedule" size="md" />
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.service_update"
              :min="1"
              :max="5"
              marker-labels
              label
            />
          </q-item-section>
        </q-item>
        <q-item-label header class="text-grey-3">任务栏更新间隔</q-item-label>
        <q-item>
          <q-item-section side>
            <q-icon color="primary" name="schedule" size="md" />
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.tray_icon_update"
              :min="1"
              :inner-min="form_value.service_update"
              :max="5"
              marker-labels
              label
            />
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section>
            <q-btn
              color="grey"
              label="放弃"
              icon="restart_alt"
              :loading="loading"
              @click="reset"
            ></q-btn>
          </q-item-section>
          <q-item-section>
            <q-btn
              color="primary"
              label="保存"
              icon="save"
              :loading="loading"
              @click="onSubmit"
            ></q-btn>
          </q-item-section>
        </q-item>
      </q-list>
    </q-form>
  </q-page>
</template>
<script setup lang="ts">
import { useQuasar } from "quasar";
import { ref } from "vue";
import { useStore as useConfig, Config } from "../stores/Config";
const $q = useQuasar();
const config_store = useConfig();
const form_value = ref(config_store.$state);
const loading = ref(false);
const onSubmit = async () => {
  loading.value = true;
  try {
    await config_store.update(form_value.value);
    $q.dialog({ message: `保存完成` });
  } catch (err) {
    $q.dialog({ message: `保存出错，错误信息:${err}` });
  }
  loading.value = false;
};
const reset = async () => {
  loading.value = true;
  const nVal = await config_store.load();
  form_value.value = nVal;
  loading.value = false;
};
</script>
