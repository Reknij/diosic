<script lang="ts" setup>
import { inject, Ref, ref, watch } from 'vue';
import { MediaInfo } from '../models';
import { getMedias, searchMedia } from '../serverApi';
import { current_media, current_source, desktopMode, tryGetCurrentSource } from './util';
import player from '../MediaPlayer';
import { Search } from '@element-plus/icons-vue';
import router from '../router';

let currentPage = ref(1);
let total = ref(0);
let getIndex = () => currentPage.value - 1;
let limit = 30;
let mediasInfo = ref<MediaInfo[]>();
let toSearch = ref('');
let loading = ref(false);

let title: Ref<string> | undefined = inject("title")

watch(router.currentRoute, async () => {
    await loadMedias()
});

function resetPages() {
    currentPage.value = 1;
    total.value = 0;
    mediasInfo.value?.splice(0);
}

async function loadMedias() {
    loading.value = true;
    resetPages();
    let query = router.currentRoute.value.query as {
        s: string,
        f: string,
    };
    await tryGetCurrentSource(query);
    if (!query.s || !query.f) return;

    if (!current_source.value) {
        current_source.value = {
            title: 'error to get current source.',
            length: 0
        };
    }
    total.value = current_source.value.length;
    mediasInfo.value = await getMedias(query.s, query.f, getIndex(), limit, loadMedias);
    if (title) title.value = `${query.s.toUpperCase()} - ${query.f}`;
    loading.value = false;
}
loadMedias();

async function clickMedia(row: MediaInfo) {
    current_media.value = row;
    await player.play(row);
}

async function changePage(newPage: number) {
    currentPage.value = newPage;
    if (!searchMode) {
        let query = router.currentRoute.value.query as {
            s: string,
            f: string,
        };
        mediasInfo.value = await getMedias(query.s, query.f, getIndex(), limit, changePage);
    }
    else {
        let result = await searchMedia(toSearch.value, getIndex(), limit, changePage);
        mediasInfo.value = result.content;
        total.value = result.length;
    }
}

let searchMode = false;
let lastVal = '';
async function searchIt(val: string) {
    if (val != '' && val != lastVal) {
        loading.value = true;
        resetPages();
        let result = await searchMedia(val, getIndex(), limit, searchIt);
        mediasInfo.value = result.content;
        total.value = result.length;
        searchMode = true;
        loading.value = false;
    }
}

async function updateSearch(input: string) {
    if (input == '') {
        await loadMedias();
        searchMode = false;
    }
    toSearch.value = input;
}

async function playAll(shuttle: boolean = false) {
    if (mediasInfo.value) {
        let query = router.currentRoute.value.query as {
            s: string,
            f: string,
        };
        let medias = await getMedias(query.s, query.f, 0, total.value, playAll);
        if (shuttle) medias = medias.sort(() => Math.random() - 0.5);
        player.playList(medias);
    }
}
</script>

<template>
    <div v-loading="loading">
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
                    <el-button round @click="playAll(false)">
                        <el-icon>
                            <svg stroke="currentColor" fill="currentColor" stroke-width="0" viewBox="0 0 512 512"
                                class="react-jinke-music-player-play-icon" height="1em" width="1em"
                                xmlns="http://www.w3.org/2000/svg">
                                <path
                                    d="M256 8C119 8 8 119 8 256s111 248 248 248 248-111 248-248S393 8 256 8zm115.7 272l-176 101c-15.8 8.8-35.7-2.5-35.7-21V152c0-18.4 19.8-29.8 35.7-21l176 107c16.4 9.2 16.4 32.9 0 42z">
                                </path>
                            </svg>
                        </el-icon>
                        <span v-if="desktopMode">Play All</span>
                    </el-button>
                    <el-button round @click="playAll(true)">
                        <el-icon>
                            <svg class="MuiSvgIcon-root jss27" focusable="false" viewBox="0 0 24 24" aria-hidden="true">
                                <path
                                    d="M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z">
                                </path>
                            </svg>
                        </el-icon>
                        <span v-if="desktopMode">Shuttle All</span>
                    </el-button>
                </el-row>
            </el-col>
        </el-row>

        <el-table :data="mediasInfo" style="width: 100%" @row-click="clickMedia">
            <el-table-column prop="title" label="Title" />
            <el-table-column prop="artist" label="Artist" />
            <el-table-column v-if="desktopMode && router.currentRoute.value.query.s != 'album'" prop="album"
                label="Album" />
            <el-table-column v-if="desktopMode" prop="genre" label="Genre" />
            <el-table-column v-if="desktopMode" prop="year" label="Year" />
            <el-table-column v-if="desktopMode && router.currentRoute.value.query.s != 'library'" prop="library"
                label="Library" />
        </el-table>
        <el-pagination class="autoMargin" :current-page="currentPage" @current-change="changePage" :page-size="limit"
            layout="total, prev, pager, next" :total="total"></el-pagination>
    </div>
</template>

<style>

</style>