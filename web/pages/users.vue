<script setup lang="ts">
import { object, string } from 'yup';
import type { UserToCreate, GetUsersQuery, UserInfo } from '~/api/model';
import { createUser, deleteUser, getUsers } from '~/api/user';

const cols = [
    {
        label: 'Id',
        key: 'id',
    },
    {
        label: 'Alias',
        key: 'alias',
    },
    {
        label: 'Username',
        key: 'username',
    },
    {
        label: 'Password',
        key: 'password',
    },
]
const query = reactive<GetUsersQuery>({
    index: 0,
    limit: 50,
})
const toast = useToast();
const isOpen = ref(false);
const state = reactive<UserToCreate>({
    username: '',
    password: '',
    alias: 'Newbee'
})
const { data: users, refresh } = await getUsers(query)
const selected = ref<UserInfo[]>([])

function select(row: UserInfo) {
    const index = selected.value.findIndex((item) => item.id === row.id)
    if (index === -1) {
        selected.value.push(row)
    } else {
        selected.value.splice(index, 1)
    }
}

const schema = object({
    alias: string().required("Required"),
    username: string().required("Required"),
    password: string().required("Required"),
})

async function registerClicked() {
    const { error } = await createUser(state);
    if (error.value) {
        toast.add({
            color: 'red',
            title: error.value.data,
        })
    } else {
        await refresh()
        toast.add({
            color: 'green',
            title: 'Add user successfully!'
        })
        isOpen.value = false;
    }
}

async function removeSelected() {
    toast.add({
        color: 'yellow',
        title: 'You sure want delete selected users?',
        actions: [
            {
                label: 'Yes',
                async click() {
                    const arr = []
                    for (let i = 0; i < selected.value.length; i++) {
                        const item = selected.value[i];
                        arr.push(deleteUser(item.username))
                    }
                    await Promise.all(arr);
                    await refresh();
                    toast.add({
                        color: 'green',
                        title: 'Delete selected users successfully!'
                    })
                }
            },
            {
                label: 'No'
            }
        ]
    })
}
</script>

<template>
    <div class="flex flex-col gap-2 p-2 h-full">
        <UModal v-model="isOpen">
            <UCard>
                <template #header>
                    <div class="flex items-center justify-between">
                        <span class="text-xl font-bold">Register user</span>
                        <UButton icon="i-mdi-close" variant="link" @click="isOpen = false" />
                    </div>
                </template>
                <UForm class="space-y-2" :schema="schema" :state="state" @keyup.enter="registerClicked">
                    <UFormGroup label="Alias">
                        <UInput v-model="state.alias" />
                    </UFormGroup>
                    <UFormGroup label="Username">
                        <UInput v-model="state.username" />
                    </UFormGroup>
                    <UFormGroup label="Password">
                        <UInput v-model="state.password" />
                    </UFormGroup>
                    <UFormGroup>
                        <UButton label="Confirm" @click="registerClicked" />
                    </UFormGroup>
                </UForm>
            </UCard>>
        </UModal>

        <UTable @select="select" v-model="selected" :rows="users?.items" :columns="cols" />
        <div class="flex-grow"></div>
        <div class="flex flex-wrap gap-2 justify-end px-3 py-3.5 border-t border-gray-200 dark:border-gray-700">
            <UButton label="Register" @click="isOpen = true" />
            <UButton label="Delete selected" @click="removeSelected" />
            <UPagination :model-value="query.index + 1" @update:model-value="v => query.index = v - 1"
                :page-count="query.limit" :total="users?.total ?? 0" />
        </div>
    </div>
</template>