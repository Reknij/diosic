<script lang="ts" setup>
import { ElMessage } from 'element-plus';
import { ref } from 'vue';
import { ServerInfo } from '../models';
import { getServerInfo, scanLibraries } from '../serverApi';
import { libraries } from './util';

let info = ref<ServerInfo>();
let time_running = ref('');
async function init() {
    info.value = await getServerInfo();
    let second = info.value.time_running;
    var days = Math.floor(second / 86400);
    var hours = Math.floor((second % 86400) / 3600);
    var minutes = Math.floor(((second % 86400) % 3600) / 60);
    var seconds = Math.floor(((second % 86400) % 3600) % 60);
    if (days > 0) {
        time_running.value = `${days} days`;
    }
    else if (hours > 0) {
        time_running.value = `${hours} hours`;
    }
    else if (minutes > 0) {
        time_running.value = `${minutes} minutes`;
    }
    else if (seconds > 0) {
        time_running.value = `${seconds} seconds`;
    }
}
init();

async function scanLibrariesClick() {
    try {
        await scanLibraries();
        ElMessage.success({
            message: "All media libraries scanned. Now refresh web page.",
            duration: 1000,
            onClose: ()=>window.location.reload(),
        })
    } catch (error: any) {
        ElMessage.error({
            message: "Scan libraries failed. Please see the message in the log of server."
        })
    }
}
</script>

<template>
    <el-tag size="large">
        Diosic Web Information
    </el-tag>
    <h4>Version: 1.1</h4>
    <h4>Author: Jinker</h4>
    <el-divider></el-divider>
    <el-tag size="large">
        Diosic Server Information
    </el-tag>
    <h4>Version: {{ info?.version }}</h4>
    <h4>Author: {{ info?.author }}</h4>
    <h4>Time running: {{ time_running }}</h4>
    <el-divider></el-divider>
    <el-tag size="large">
        Libraries
    </el-tag>
    <ul>
        <li v-for="(lib, i) in libraries" :key="i">{{ lib.title }} ({{ lib.length }} medias)</li>
    </ul>
    <el-row align="middle">
        <h4>Count: {{ libraries?.length }}</h4>
        <el-button @click="scanLibrariesClick" style="margin-left: 10px;">Scan libraries</el-button>
    </el-row>
</template>