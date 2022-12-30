<script setup lang="ts">
import Cookies from 'js-cookie';
import { provide, ref, watch } from 'vue';
import { current_user } from './components/util';
import { UserInfo } from './models';
import router from './router';
import { getCurrentUser, requireSetup } from './serverApi';

async function autoRedirect() {
    if (router.currentRoute.value.path == '/') await routeTo();
}

watch(router.currentRoute, autoRedirect);

async function routeTo() {
    if (await requireSetup()) {
        await router.replace('/setup')
    }
    else if (!Cookies.get("authorization")) {
        await router.replace('/login');
    }
    else {
        try {
            current_user.value = await getCurrentUser();
            if (router.currentRoute.value.path == '/') await router.replace('/home');
        } catch (error) {
            Cookies.remove('authorization')
            await router.replace('/login');
        }
    }
}
routeTo();
</script>

<template>
    <router-view></router-view>
</template>

<style>
.autoMargin {
    margin-top: 10px;
    margin-bottom: 10px;
}

body {
    margin: 0;
}

div,
input,
textarea,
button,
select,
a {
    -webkit-tap-highlight-color: rgba(0, 0, 0, 0);
}
</style>
