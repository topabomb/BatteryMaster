// FILE: main.js

import { createApp } from "vue";
import { Quasar, Notify, Dialog } from "quasar";
import quasarLang from "quasar/lang/zh-CN";
import router from "./router";
import { createPinia } from "pinia";

// Import icon libraries
import "@quasar/extras/material-icons/material-icons.css";
import "@quasar/extras/material-symbols-outlined/material-symbols-outlined.css";

// Import Quasar css
import "quasar/src/css/index.sass";

// Assumes your root component is App.vue
// and placed in same folder as main.js
import App from "./App.vue";

import { zhCN } from "date-fns/locale";
import { setDefaultOptions } from "date-fns";
setDefaultOptions({ locale: zhCN });

const myApp = createApp(App);

myApp.use(Quasar, {
  plugins: { Notify, Dialog }, // import Quasar plugins and add here
  lang: quasarLang,
  /*
  config: {
    brand: {
      // primary: '#e46262',
      // ... or all other brand colors
    },
    notify: {...}, // default set of options for Notify Quasar plugin
    loading: {...}, // default set of options for Loading Quasar plugin
    loadingBar: { ... }, // settings for LoadingBar Quasar plugin
    // ..and many more (check Installation card on each Quasar component/directive/plugin)
  }
  */
});

// Assumes you have a <div id="app"></div> in your index.html
const pinia = createPinia();
myApp.use(router);
myApp.use(pinia);
myApp.mount("#app");
