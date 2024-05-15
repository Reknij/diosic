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
const isGuestEnter = ref(false);

async function loginBtnClicked() {
    await loginNow(state);
    router.replace('/');
}
async function guestEnter() {
    isGuestEnter.value = true;
    state.username = 'guest';
    state.password = '';
    if (serverInfo.value?.guest_password_required === false) {
        await loginBtnClicked();
    }
}

function userEnter() {
    isGuestEnter.value = false;
    state.username = '';
    state.password = '';
}
</script>

<template>
    <div class="flex items-center justify-center p-2 h-full">
        <UCard>
            <UForm :schema="schema" :state="state" class="space-y-2" @keyup.enter="loginBtnClicked">
                <UFormGroup label="Username" v-if="!isGuestEnter">
                    <UInput v-model="state.username" />
                </UFormGroup>
                <UFormGroup label="Password" v-if="!isGuestEnter || serverInfo?.guest_password_required">
                    <UInput v-model="state.password" />
                </UFormGroup>
                <UFormGroup v-if="!isGuestEnter">
                    <div class="flex gap-2">
                        <UButton @click="loginBtnClicked" label="Login" />
                        <UButton v-if="serverInfo?.guest_enable" @click="guestEnter" label="Guest enter" />
                    </div>
                </UFormGroup>
                <UFormGroup v-else>
                    <UButton @click="userEnter" label="User enter" />
                </UFormGroup>
            </UForm>
        </UCard>
    </div>
</template>