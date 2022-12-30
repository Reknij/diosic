<script lang="ts" setup>
import { provide, ref, watch } from 'vue'
import MyMenu from './MyMenu.vue';
import { current_source, desktopMode, tryGetCurrentSource } from "./util";
import { Expand } from '@element-plus/icons-vue';
import { getLibraries } from '../serverApi';
import router from '../router';

let drawer = ref(false);
let title = ref("Diosic Web");

let scrollbarHeight = ref(window.innerHeight - 60);
window.onresize = e => {
    scrollbarHeight.value = window.innerHeight - 60;
}

async function init() {
    await tryGetCurrentSource(router.currentRoute.value.query);

    if (!current_source.value) {
        let libraries = await getLibraries();
        if (libraries.length) {
            current_source.value = libraries[0];
            await router.replace({
                path: '/home/medias',
                query: {
                    s: 'library',
                    f: libraries[0].title
                }
            })
        }
    }
}
init();

provide("title", title);
</script>

<template>
    <el-container>
        <el-aside v-if="desktopMode">
            <MyMenu></MyMenu>
        </el-aside>
        <el-drawer size="50%" direction="ltr" v-else :with-header="false" v-model="drawer">
            <MyMenu style="border: none;"></MyMenu>
        </el-drawer>
        <el-container>
            <el-header height="60px">
                <el-row align="middle">
                    <el-button v-if="!desktopMode" :icon="Expand" @click="drawer = true"></el-button>
                    <el-avatar :size="32" src="/diosic.svg" style="margin-left: 10px;"></el-avatar>
                    <h1 style="margin-left: 15px;">{{ title }}</h1>
                </el-row>
            </el-header>
            <el-scrollbar :height="scrollbarHeight">
                <el-main>
                    <p v-if="router.currentRoute.value.path == '/home' || router.currentRoute.value.path == '/home/'">
                        Please ensure you already settle all library.
                    </p>
                    <router-view></router-view>
                </el-main>
                <el-backtop :right="25" :bottom="50" target=".el-scrollbar__wrap" />
            </el-scrollbar>
        </el-container>
    </el-container>
</template>

<style>
.el-main {
    background-color: rgb(36, 36, 36);
    min-height: calc(100vh - 60px);
}
</style>