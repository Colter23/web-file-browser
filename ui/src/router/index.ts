import { createRouter, createWebHistory } from 'vue-router'
import MainView from "../views/MainView.vue";
import {getSession} from "../network/auth-api";


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

router.beforeEach(async (to) => {
  if (to.path === '/login') return true;

  try {
    const session = await getSession();
    if (session.authenticated) return true;
  } catch (error) {
    console.error(error);
  }

  return {
    path: '/login',
    query: {redirect: to.fullPath}
  };
})

export default router
