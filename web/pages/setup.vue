<script setup lang="ts">
import type { ToSetup } from '~/api/model';
import { object, string } from 'yup'
import { getServerInfo, setup } from '~/api/server';
import { useServerInfo } from '~/composables/serverInfo';

const router = useRouter();
const currentUser = useCurrentUser();
const serverInfo = useServerInfo();

if (!serverInfo.value?.admin_required && !currentUser.value?.is_admin) {
    router.push('/')
}

const schema = object({
    alias: string().required("Required"),
    username: string().required("Required"),
    password: string().required("Required"),
    guestEnable: string().required("Required"),
    guestPassword: string().nullable(),
})
const state = reactive<ToSetup>({
    alias: currentUser.value?.alias ?? 'Admin',
    username: currentUser.value?.username ?? 'admin',
    password: currentUser.value?.password ?? '',
    guest_enable: serverInfo.value?.guest_enable ?? false,
})

async function setupNow() {
    const { error } = await setup(state);
    if (error.value) {
        console.error(error.value);
    } else {
        const { data } = await getServerInfo();
        if (data.value) {
            useServerInfo().value = data.value;
        }
        await logoutNow();
        await router.push('/login')
    }
}
</script>

<template>
    <div class="flex items-center justify-center p-2 h-full">
        <UCard>
            <UForm :schema="schema" :state="state" class="space-y-2" @keyup.enter="setupNow">
                <UFormGroup label="Alias">
                    <UInput v-model="state.alias" />
                </UFormGroup>
                <UFormGroup label="Username">
                    <UInput v-model="state.username" />
                </UFormGroup>
                <UFormGroup label="Password">
                    <UInput v-model="state.password" />
                </UFormGroup>
                <UFormGroup label="Guest enable">
                    <div class="flex flex-col gap-2">
                        <UToggle v-model="state.guest_enable" />
                        <UInput v-if="state.guest_enable" placeholder="Optional" v-model="state.guest_password" />
                    </div>
                </UFormGroup>
                <UFormGroup>
                    <UButton label="Confirm" @click="setupNow" />
                </UFormGroup>
            </UForm>
        </UCard>
    </div>
</template>