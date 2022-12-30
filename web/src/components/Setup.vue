<script lang="ts" setup>
import { ElMessage } from "element-plus";
import Cookies from "js-cookie";
import { ref } from "vue";
import { UserInfo } from "../models";
import router from "../router";
import { createUser, login } from "../serverApi";
import { current_user } from "./util";

let toCreate = ref<UserInfo>({
    username: 'admin',
    password: '12345678',
    alias: 'Admin',
    is_admin: true
})
let now = ref('Admin account')
async function register() {
    try {
        if (!toCreate.value.password) return;
        await createUser(toCreate.value);
        let logined = await login(toCreate.value.username, toCreate.value.password);
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
        <el-form id="setupForm" v-model="toCreate" @keyup.enter="register">
            <el-row justify="center">
                <el-avatar :size="256" src="/diosic.svg"></el-avatar>
            </el-row>
            <h1>Diosic Setup - {{ now }}</h1>
            <el-form-item label="Alias">
                <el-input v-model="toCreate.alias"></el-input>
            </el-form-item>
            <el-form-item label="Username">
                <el-input v-model="toCreate.username"></el-input>
            </el-form-item>
            <el-form-item label="Password">
                <el-input show-password v-model="toCreate.password"></el-input>
            </el-form-item>
            <el-form-item>
                <el-button type="primary" @click="register">Register</el-button>
            </el-form-item>
        </el-form>
    </el-row>
</template>

<style>

</style>