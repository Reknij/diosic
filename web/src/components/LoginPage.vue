<script setup lang="ts">
import Cookies from 'js-cookie';
import { inject, reactive, ref } from 'vue'
import { UserInfo } from '../models';
import { login } from '../serverApi';
import router from '../router';
import { current_user, getSourcesInfo, setup_info } from './util';
import { ElMessage } from 'element-plus';

let guestMode = ref(false)
let form = reactive({
    username: '',
    password: '',
})

async function guestAccess() {
    guestMode.value = !guestMode.value;
    form.password = '';
    if (!setup_info.value?.guest_password_required) {
        await justLogin(true);
    }
}

async function loginClick() {
    if (guestMode.value) {
        await justLogin(true);
        return;
    }
    if (!form.username) {
        ElMessage.error({
            message: "Please enter your username to continue."
        })
        return;
    }
    if (!form.password) {
        ElMessage.error({
            message: "Please enter your password to continue."
        })
        return;
    }
    await justLogin()
}

async function justLogin(guest = false) {
    try {
        let lu = await login(guest? "guest": form.username, form.password);
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
            <el-form-item v-show="!guestMode" label="Username">
                <el-input v-model="form.username"></el-input>
            </el-form-item>
            <el-form-item v-if="!guestMode || setup_info?.guest_password_required" label="Password">
                <el-input show-password v-model="form.password"></el-input>
            </el-form-item>
            <el-form-item>
                <el-button v-if="setup_info?.guest_enable" type="primary" @click="guestAccess">{{ guestMode ? `User access`: `Guest access` }}</el-button>
                <el-button v-if="!guestMode || setup_info?.guest_password_required" type="primary"
                    @click="loginClick">Login</el-button>
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
