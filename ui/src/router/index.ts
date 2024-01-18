import { createRouter, createWebHistory } from 'vue-router'
import MainView from "../views/MainView.vue";


const routes = [
  {
    path: '/',
    name: 'Main',
    component: MainView
  },
  {
    path: '/login',
    name: 'Login',
    component: () => import('../views/LoginView.vue')
  },
  {
    path: '/setting',
    name: 'Setting',
    component: () => import('../views/SettingView.vue')
  }
]


const router = createRouter({
  history: createWebHistory(),
  routes
})


export default router
