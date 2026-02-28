import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('../views/Dashboard.vue'),
  },
  {
    path: '/upload',
    name: 'Upload',
    component: () => import('../views/Upload.vue'),
  },
  {
    path: '/batch/:id',
    name: 'BatchDetail',
    component: () => import('../views/BatchDetail.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('../views/Settings.vue'),
  },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})
