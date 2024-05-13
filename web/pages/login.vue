<script setup lang="ts">
import type { LoginQuery } from '~/api/model';
import { object, string } from 'yup'
import { useServerInfo } from '~/composables/serverInfo';

const router = useRouter();
const serverInfo = useServerInfo();
const schema = object({
    username: string().required("Required"),
    password: string().required("Required"),
})
const state = reactive<LoginQuery>({
    username: '',
    password: '',
})

async function loginBtnClicked() {
    await loginNow(state);
    router.replace('/');
}
async function guestEnter() {
    state.username = 'guest';
    state.password = '';
    if (serverInfo.value?.guest_password_required === false) {
        await loginBtnClicked();
    }
}
</script>

<template>
    <div class="flex items-center justify-center p-2 h-full">
        <UCard>
            <UForm :schema="schema" :state="state" class="space-y-2" @keyup.enter="loginBtnClicked">
                <UFormGroup label="Username" v-if="!serverInfo?.guest_enable && state.username !== 'guest'">
                    <UInput v-model="state.username" />
                </UFormGroup>
                <UFormGroup label="Password" v-if="state.username !== 'guest' || serverInfo?.guest_password_required">
                    <UInput v-model="state.password" />
                </UFormGroup>
                <UFormGroup v-if="state.username !== 'guest'">
                    <div class="flex gap-2">
                        <UButton @click="loginBtnClicked" label="Login" />
                        <UButton @click="guestEnter" label="Guest enter" />
                    </div>
                </UFormGroup>
                <UFormGroup v-else>
                    <UButton @click="state.username = ''" label="User enter" />
                </UFormGroup>
            </UForm>
        </UCard>
    </div>
</template>