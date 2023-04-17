<script lang="ts" setup>
import { inject, reactive, Ref, ref } from 'vue';
import router from '../router';
import { updateUser } from '../serverApi';
import { current_user, setup_info } from './util';

let title = inject<Ref<string>>('title')
let form = ref(current_user.value);
let isGuest = ref(form.value?.username == "guest");
async function apply() {
    if (form.value) {
        await updateUser(form.value);
        current_user.value = form.value;
        await router.replace('/');
    }
}

if (title) {
    title.value = 'Personal'
}
</script>

<template>
    <el-row v-if="form" align="middle" justify="center">
        <el-form class="loginForm" v-model="form">
            <h1 v-if="!isGuest">
                Update information
            </h1>
            <el-form-item label="Username">
                <el-input v-model="form.username" disabled></el-input>
            </el-form-item>
            <el-form-item label="Alias">
                <el-input v-model="form.alias" :disabled="isGuest"></el-input>
            </el-form-item>
            <el-form-item label="Password">
                <el-input show-password v-model="form.password" :disabled="isGuest"></el-input>
            </el-form-item>
            <el-form-item>
                <el-button type="primary" @click="apply" :disabled="isGuest">Apply</el-button>
            </el-form-item>
        </el-form>
    </el-row>
</template>