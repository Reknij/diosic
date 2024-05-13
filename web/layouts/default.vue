<script setup lang="ts">
import { useMediaPlayerState } from '~/composables/player';


const playerState = useMediaPlayerState();
</script>

<template>
    <div class="flex flex-col min-h-screen">
        <MyHeader />
        <div class="flex-1 bg-gray-100 dark:bg-slate-800">
                <slot></slot>
        </div>
        <MediaPlayer class="flex sticky bottom-0 z-10" :state="playerState"/>
    </div>
</template>