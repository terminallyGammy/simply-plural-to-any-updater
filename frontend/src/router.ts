
import { createRouter, createWebHistory } from 'vue-router';
import Home from './components/Home.vue';
import Config from './components/Config.vue';

const routes = [
  { path: '/', component: Home },
  { path: '/config', component: Config },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
