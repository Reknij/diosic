<script setup lang="ts">
import Cookies from 'js-cookie';
import { inject, reactive, Ref } from 'vue'
import { UserInfo } from '../models';
import { login } from '../serverApi';
import router from '../router';
import { current_user, getSourcesInfo } from './util';
import { ElMessage } from 'element-plus';

let form = reactive({
    username: '',
    password: '',
})

async function loginClick() {
    try {
        if (!form.username || !form.password) {
            ElMessage.error({
                message: "Please enter your username and password to continue."
            })
            return;
        }

        let lu = await login(form.username, form.password);
        current_user.value = lu.current;
        Cookies.set('authorization', lu.token.toString(), {
            expires: 180,
        });
        await getSourcesInfo()
        router.replace('/home')
    }
    catch (err: any) {
        ElMessage.error({
            message: `Failed verify, please ensure your username and password is correctly! (${err.response.data})`
        })
    }
}

</script>

<template>
    <el-row align="middle" justify="center" style="min-height: 100vh;">
        <el-form class="loginForm" v-model="form" @keyup.enter="loginClick">
            <el-row justify="center">
                <el-avatar :size="256" src="/diosic.svg"></el-avatar>
            </el-row>
            <h1>
                Login to Diosic
            </h1>
            <el-form-item label="Username">
                <el-input v-model="form.username"></el-input>
            </el-form-item>
            <el-form-item label="Password">
                <el-input show-password v-model="form.password"></el-input>
            </el-form-item>
            <el-form-item>
                <el-button type="primary" @click="loginClick">Login</el-button>
            </el-form-item>
        </el-form>
    </el-row>
</template>

<style>
#loginForm {
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;
}
</style>
