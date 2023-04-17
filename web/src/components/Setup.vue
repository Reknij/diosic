<script lang="ts" setup>
import { ElMessage } from "element-plus";
import Cookies from "js-cookie";
import { ref } from "vue";
import { UserInfo, ToSetup } from "../models";
import router from "../router";
import { setup, login } from "../serverApi";
import { current_user } from "./util";

let toSetup = ref<ToSetup>({
    admin: {
        username: 'admin',
        password: '12345678',
        alias: 'Admin',
        is_admin: true
    },
    guest_enable: false,
    guest_password: undefined,
})
let now = ref('Admin account')
async function setupAll() {
    try {
        if (!toSetup.value.admin.password) return;
        await setup(toSetup.value);
        let logined = await login(toSetup.value.admin.username, toSetup.value.admin.password);
        if (logined) {
            current_user.value = logined.current;
            Cookies.set('authorization', logined.token);
        }
        await router.push('/home')
    } catch (error: any) {
        ElMessage.error({
            message: error.response.data
        })
    }

}
</script>

<template>
    <el-row align="middle" justify="center" style="min-height: 100vh; margin-left: 10px; margin-right: 10px;">
        <el-form id="setupForm" v-model="toSetup" @keyup.enter="setupAll">
            <el-row justify="center">
                <el-avatar :size="256" src="/diosic.svg"></el-avatar>
            </el-row>
            <h1>Diosic Setup - {{ now }}</h1>
            <el-form-item label="Alias">
                <el-input v-model="toSetup.admin.alias"></el-input>
            </el-form-item>
            <el-form-item label="Username">
                <el-input v-model="toSetup.admin.username"></el-input>
            </el-form-item>
            <el-form-item label="Password">
                <el-input show-password v-model="toSetup.admin.password"></el-input>
            </el-form-item>
            <el-form-item label="Guest enable">
                <el-switch v-model="toSetup.guest_enable"></el-switch>
            </el-form-item>
            <el-form-item v-show="toSetup.guest_enable" label="Guest password (Optional)">
                <el-input v-model="toSetup.guest_password"></el-input>
            </el-form-item>
            <el-form-item>
                <el-button type="primary" @click="setupAll">Register</el-button>
            </el-form-item>
        </el-form>
    </el-row>
</template>

<style></style>