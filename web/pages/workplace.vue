<script setup lang="ts">
import { getMedias } from '~/api/media';
import { Source } from '~/api/model';
import { reloadMedias, reloadPlugins } from '~/api/server';
import { useServerInfo } from '~/composables/serverInfo';

const toast = useToast();
const serverInfo = useServerInfo();
const { data: medias } = await getMedias({ index: 0, limit: 0, source: Source.Any });

async function reloadMediasHandler() {
    const { error } = await reloadMedias()
    if (error.value) {
        toast.add({
            color: 'red',
            title: error.value.data
        })
    } else {
        toast.add({
            color: 'green',
            title: 'Reload medias successfully!'
        })
    }
}

async function reloadPluginsHandler() {
    const { error } = await reloadPlugins()
    if (error.value) {
        toast.add({
            color: 'red',
            title: error.value.data
        })
    } else {
        toast.add({
            color: 'green',
            title: 'Reload plugins successfully!'
        })
    }
}

async function reloadAllHandler() {
    await reloadPluginsHandler();
    await reloadMediasHandler();
}
</script>

<template>
    <div class="flex flex-col items-center justify-center gap-2 p-2">
        <div class="flex flex-col gap-2 w-fit">
            <span class="text-xl font-bold mx-auto">Actions</span>
            <UButton label="Reload medias" variant="outline" @click="reloadMediasHandler" />
            <UButton label="Reload plugins" variant="outline" @click="reloadPluginsHandler" />
            <UButton label="Reload all" @click="reloadAllHandler" />
            <UButton label="Setup again" to="/setup" />
            <UDivider orientation="vertical" class="w-full text-white" :ui="{ container: { base: 'text-black' } }" />
            <span>Total media: {{ medias?.total ?? 0 }}</span>
            <span>Server time running: {{ serverInfo?.time_running ? (serverInfo.time_running / 60 / 60).toFixed(1) + ' hour(s)' : 'Just started' }}</span>
            <span>Author: {{ serverInfo?.author }}</span>
            <span>Version: {{ serverInfo?.version }}</span>
        </div>
    </div>
</template>