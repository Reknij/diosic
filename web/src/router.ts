import * as VueRouter from 'vue-router'
import LoginPage from './components/LoginPage.vue'
import Setup from './components/Setup.vue'
import Medias from './components/Medias.vue'
import Home from './components/Home.vue'
import Dashboard from './components/Dashboard.vue'
import Personal from './components/Personal.vue'
import Users from './components/Users.vue'

const routes = [
    {
        path: '/',
        children: [
            {
                path: '/home',
                component: Home,
                children: [
                    {
                        path: 'dashboard',
                        component: Dashboard,
                    },
                    {
                        path: 'medias',
                        component: Medias
                    },
                    {
                        path: 'personal',
                        component: Personal
                    },
                    {
                        path: 'users',
                        component: Users
                    }
                ]
            },
            {
                path: '/login',
                component: LoginPage
            },
            {
                path: '/setup',
                component: Setup
            },
        ]
    },
]

// 3. Create the router instance and pass the `routes` option
// You can pass in additional options here, but let's
// keep it simple for now.
const router = VueRouter.createRouter({
    // 4. Provide the history implementation to use. We are using the hash history for simplicity here.
    history: VueRouter.createWebHistory(),
    routes, // short for `routes: routes`
})

export default router;