<script setup lang="ts">
import type { UserToCreate } from '~/api/model';
import { object, string } from 'yup'
import { createUser } from '~/api/user';

const router = useRouter();
const schema = object({
    alias: string().required("Required"),
    username: string().required("Required"),
    password: string().required("Required"),
})
const state = reactive<UserToCreate>({
    alias: '',
    username: '',
    password: '',
})

async function registerBtnClicked() {
    await createUser(state).then(() => router.replace('/login')).catch(err => console.log(err));
}
</script>

<template>
    <div class="flex items-center justify-center">
        <UCard>
            <UForm :schema="schema" :state="state" class="space-y-2" @keyup.enter="registerBtnClicked">
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
                    <UButton @click="registerBtnClicked" label="Register" />
                </UFormGroup>
            </UForm>
        </UCard>
    </div>
</template>