<script lang="ts" setup>
import { Delete, Search } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { ref } from 'vue';
import { UserInfo } from '../models';
import { createUser, deleteUser, getUsers, searchUser } from '../serverApi';

let usersInfo = ref<UserInfo[]>();

let currentPage = ref(1);
let total = ref(0);
let getIndex = () => currentPage.value - 1;
let limit = 30;
let toSearch = ref('');

function resetPages() {
    currentPage.value = 1;
}

async function loadUsers() {
    resetPages();
    usersInfo.value = await getUsers(getIndex(), limit);
    total.value = usersInfo.value.length;
}
loadUsers();

async function changePage(newPage: number) {
    currentPage.value = newPage;
    usersInfo.value = await getUsers(getIndex(), limit);
    total.value = usersInfo.value.length;
}

let searchMode = false;
let lastVal = '';
async function searchIt(val: string) {
    if (val != '' && val != lastVal) {
        resetPages();
        let result = await searchUser(val, getIndex(), limit);
        usersInfo.value = result.content;
        total.value = result.length;
        searchMode = true;
    }
}

async function updateSearch(input: string) {
    if (input == '') {
        await loadUsers();
        searchMode = false;
    }
    toSearch.value = input;
}

async function goRegister() {
    ElMessageBox.prompt('Please input the username', 'What username?', {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        inputPattern:
            /.+/,
        inputErrorMessage: 'Invalid username',
    })
        .then(({ value }) => {
            let username = value;
            ElMessageBox.prompt('Please input the password', 'WHat password?', {
                confirmButtonText: 'OK',
                cancelButtonText: 'Cancel',
                inputPattern:
                    /.+/,
                inputErrorMessage: 'Invalid password',
            })
                .then(async ({ value }) => {
                    let password = value;
                    try {
                        let user: UserInfo = {
                            username,
                            password,
                            alias: username,
                            is_admin: false,
                        };
                        await createUser(user);
                        usersInfo.value?.push(user);
                        ElMessage.success({
                            message: `Success create user \`${username}\``
                        })
                    } catch (error: any) {
                        ElMessage.error({
                            message: error.response.data,
                        })
                    }
                })
        })
}

async function deleteUserClick(row: any) {
    try {
        await deleteUser(row.username);
        if (usersInfo.value) {
            usersInfo.value.splice(usersInfo.value.indexOf(row), 1);
        }
    } catch (error: any) {
        ElMessage.error({
            message: error.response.data,
        })
    }
}
</script>

<template>
    <el-row :gutter="8" justify="space-between">
        <el-col :span="12">
            <el-input style="margin-bottom: 10px;" :model-value="toSearch" @update:model-value="updateSearch"
                @change="searchIt" placeholder="Search" clearable>
                <template #append>
                    <el-button :icon="Search"></el-button>
                </template>
            </el-input>
        </el-col>
        <el-col :span="12">
            <el-row justify="end">
                <el-button round @click="goRegister">
                    <el-icon>
                        <svg stroke="currentColor" fill="currentColor" stroke-width="0" viewBox="0 0 512 512"
                            class="react-jinke-music-player-play-icon" height="1em" width="1em"
                            xmlns="http://www.w3.org/2000/svg">
                            <path
                                d="M256 8C119 8 8 119 8 256s111 248 248 248 248-111 248-248S393 8 256 8zm115.7 272l-176 101c-15.8 8.8-35.7-2.5-35.7-21V152c0-18.4 19.8-29.8 35.7-21l176 107c16.4 9.2 16.4 32.9 0 42z">
                            </path>
                        </svg>
                    </el-icon>
                    <span>Create User</span>
                </el-button>
            </el-row>
        </el-col>
    </el-row>

    <el-table :data="usersInfo" style="width: 100%">
        <el-table-column label="Username">
            <template #default="scope">
                <el-tag v-if="scope.row.is_admin" size="large">{{ scope.row.username }}</el-tag>
                <span v-else>{{ scope.row.username }}</span>
            </template>
        </el-table-column>
        <el-table-column prop="alias" label="Alias" />
        <el-table-column label="Password">
            <template #default="scope">
                <el-input type="password" show-password :model-value="scope.row.password"></el-input>
            </template>
        </el-table-column>
        <el-table-column label="Controls">
            <template #default="scope">
                <el-button :icon="Delete" round @click="deleteUserClick(scope.row)"></el-button>
            </template>
        </el-table-column>
    </el-table>
    <el-pagination class="autoMargin" :current-page="currentPage" @current-change="changePage" :page-size="limit"
        layout="total, prev, pager, next" :total="total"></el-pagination>
</template>