<script setup lang="ts">
import Cookies from "js-cookie";
import { ref } from "vue";
import { MediaSourceInfo } from '../models'
import router from "../router";
import { current_source, libraries, albums, categories, artists, genres, years, current_user } from "./util";
import { logout, scanLibraries } from '../serverApi';
import { ElMessage, ElMessageBox } from 'element-plus'

async function go(source: string, current: MediaSourceInfo) {
    current_source.value = current;
    await router.push({
        path: '/home/medias',
        query: {
            s: source,
            f: current.title
        }
    })
}

let defaultActive = ref('library');
let defaultOpendeds = ref(['library']);
let query = router.currentRoute.value.query;
if (query.s) {
    if (query.f) defaultActive.value = `${query.s}-${query.f}`;
    else defaultActive.value = `${query.s}`;
}

async function logoutNow() {
    ElMessageBox.confirm(
        'Are you sure you want to log out?',
        'Log out',
        {
            confirmButtonText: 'Yes',
            cancelButtonText: 'No',
            type: 'warning',
        }
    )
        .then(async () => {
            let auth = Cookies.get('authorization');
            if (auth) {
                Cookies.remove('authorization');
                await logout(auth);
                await router.replace('/');
            }

            ElMessage({
                type: 'success',
                message: 'Logged out',
            })
        })
        .catch(() => {
            // user click `No`.
        })
}

async function toDashboard() {
    await router.push('/home/dashboard');
}

async function toPersonal() {
    await router.push('/home/personal');
}

async function toUsers() {
    await router.push('/home/users');
}
</script>

<template>
    <el-menu v-model:default-active="defaultActive" :default-openeds="defaultOpendeds" unique-opened>
        <el-sub-menu index="library">
            <template #title>
                <span>Libraries</span>
            </template>
            <el-menu-item :index="`library-${val.title}`" :key="i" v-for="(val, i) in libraries"
                @click="go('library', val)">
                {{ val.title }}</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="album">
            <template #title>
                <span>Albums</span>
            </template>
            <el-menu-item :index="`album-${val.title}`" :key="i" v-for="(val, i) in albums" @click="go('album', val)">
                {{ val.title }}</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="category">
            <template #title>
                <span>Categories</span>
            </template>
            <el-menu-item :index="`category-${val.title}`" :key="i" v-for="(val, i) in categories"
                @click="go('category', val)">
                {{ val.title }}</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="artist">
            <template #title>
                <span>Artists</span>
            </template>
            <el-menu-item :index="`artist-${val.title}`" :key="i" v-for="(val, i) in artists"
                @click="go('artist', val)">
                {{ val.title }}</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="genre">
            <template #title>
                <span>Genres</span>
            </template>
            <el-menu-item :index="`genre-${val.title}`" :key="i" v-for="(val, i) in genres" @click="go('genre', val)">
                {{ val.title }}</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="year">
            <template #title>
                <span>Years</span>
            </template>
            <el-menu-item :index="`year-${val.title}`" :key="i" v-for="(val, i) in years" @click="go('year', val)">
                {{ val.title }}</el-menu-item>
        </el-sub-menu>
        <el-divider></el-divider>

        <el-sub-menu index="control">
            <template #title>
                <span>Control Panel</span>
            </template>
            <el-menu-item v-if="current_user?.is_admin" index="control-dashboard"
                @click="toDashboard">Dashboard</el-menu-item>
            <el-menu-item index="control-personal" @click="toPersonal">Personal</el-menu-item>
            <el-menu-item v-if="current_user?.is_admin" index="control-users" @click="toUsers">Users</el-menu-item>
            <el-menu-item index="control-logout" @click="logoutNow">Log out</el-menu-item>
        </el-sub-menu>
    </el-menu>
</template>

<style>

</style>
