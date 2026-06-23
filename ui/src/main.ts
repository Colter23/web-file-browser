import { createApp } from "vue"
import App from "./App.vue"
import router from "./router";
import {createPinia} from "pinia";
import editorConfig from "./assets/editor-config.json"

import "./style.css"
import {useFileStore} from "./store";


createApp(App).use(router).use(createPinia()).mount('#app')

const fileStore = useFileStore();

editorConfig.mode.forEach((mode) => {
    if (mode.extensions != undefined) {
        fileStore.extensions.push(...mode.extensions)
    }
})
