<script setup lang="ts">
import { getMedias } from '~/api/media';
import type { GetMediasQuery, PubMediaInfo, Source } from '~/api/model';
import { PlayMode, useMediaPlayer, useMediaPlayerState } from '~/composables/player';

const props = defineProps<{
    source: Source,
    filter?: string,
}>();
const toast = useToast();
const searchValue = ref('')
const query = reactive<GetMediasQuery>({
    index: 0,
    limit: 40,
    source: props.source,
    filter: props.filter,
    to_search: searchValue.value,
})
const { data: medias } = await getMedias(query);

async function searchClicked() {
    query.index = 0;
    query.to_search = searchValue.value;
}
const player = useMediaPlayer();
const state = useMediaPlayerState();

async function playAll(mode: PlayMode) {
    const { data, error } = await getMedias({
        ...query,
        index: 0,
        limit: 99999999,
    })
    if (error.value) {
        toast.add({
            color: 'red',
            title: error.value.data
        })
    } else if (data.value) {
        player.playList(data.value.items, mode);
        player.show();
    }
}

function getMoreActions(media: PubMediaInfo) {
    return [
        [
            {
                icon: 'i-mdi-play',
                label: 'Play',
                click() {
                    player.play(media);
                }
            },
            {
                icon: 'i-mdi-motion-play-outline',
                label: 'Play next',
                click() {
                    player.push(media, state.value.currentIndex + 1)
                    if (!state.value.current && !state.value.playing) {
                        player.skipTo(media);
                    }
                    player.show();
                }
            },
            {
                icon: 'i-mdi-playlist-plus',
                label: 'Add to playlist',
                click() {
                    player.push(media);
                    player.show();
                }
            },
            {
                icon: 'i-heroicons-photo',
                label: 'Open cover',
                disabled: !media.cover_url,
                click() {
                    window.open(media.cover_url, '_blank');
                }
            }
        ]
    ]
}
</script>

<template>
    <div class="flex flex-col gap-2">
        <div class="flex gap-2 items-center flex-wrap">
            <UInput class="md:max-w-60" :ui="{ rounded: 'rounded-2xl', icon: { trailing: { pointer: '' } } }"
                v-model:model-value="searchValue" size="sm" placeholder="Search.." @keyup.enter="searchClicked">
                <template #trailing>
                    <UButton as="span" icon="i-heroicons-magnifying-glass-20-solid" @click="searchClicked"
                        color="primary" variant="link" :padded="false" />
                </template>
            </UInput>
            <div class="flex gap-2 items-center">
                <UButton icon="i-mdi-playlist-play" label="All" @click="playAll(PlayMode.Order)" />
                <UButton icon="i-mdi-repeat" label="Repeat" @click="playAll(PlayMode.Repeat)" />
                <UButton icon="i-mdi-shuffle-variant" label="Shuffle" @click="playAll(PlayMode.Shuffle)" />
            </div>
        </div>

        <div
            class="grid grid-flow-row auto-rows-fr grid-cols-2 min-[410px]:grid-cols-3 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 2xl:grid-cols-9 min-[1920px]:grid-cols-12 gap-2 justify-items-center">
            <div class="relative wild-card wild-card-btn hover:!cursor-default flex flex-col items-center justify-center !size-full"
                v-for="media in medias?.items">
                <div class="relative flex items-center justify-center size-full">
                    <img loading="lazy" class="size-full rounded object-cover" v-if="media.cover_url"
                        :src="setAuthQuery(media.cover_url)" :onerror="() => {
                            media.cover_url = undefined;
                        }">
                    <svg v-else class="text-gray-300 size-full rounded object-cover" xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24">
                        <path fill="currentColor"
                            d="M16 9h-3v5.5a2.5 2.5 0 0 1-2.5 2.5A2.5 2.5 0 0 1 8 14.5a2.5 2.5 0 0 1 2.5-2.5c.57 0 1.08.19 1.5.5V7h4zm-4-7a10 10 0 0 1 10 10a10 10 0 0 1-10 10A10 10 0 0 1 2 12A10 10 0 0 1 12 2m0 2a8 8 0 0 0-8 8a8 8 0 0 0 8 8a8 8 0 0 0 8-8a8 8 0 0 0-8-8" />
                    </svg>

                    <div class="media-overlay">
                        <div class="flex items-center justify-center w-full h-full">
                            <UButton icon="i-mdi-play-circle" variant="link" size="xl"
                                @click="player.play(media, true)" />
                            <UDropdown :items="getMoreActions(media)" :popper="{ placement: 'bottom-start' }">
                                <UButton icon="i-mdi-dots-vertical" size="xl" variant="link" />
                            </UDropdown>
                        </div>
                    </div>
                </div>
                <div :class="state.current?.id === media.id ? `w-full text-primary` : `w-full`">
                    <div class="flex flex-col justify-start p-2">
                        <div class="flex flex-row items-center gap-1">
                            <div v-if="state.current?.id === media.id" class="flex flex-row items-center gap-1">
                                <UIcon v-if="state.playing" name="i-mdi-music animate-bounce" />
                                <UIcon v-else name="i-mdi-pause" />
                            </div>
                            <UTooltip :text="media.title">
                                <span class="font-bold text-sm line-clamp-1">{{ media.title }}</span>
                            </UTooltip>
                        </div>
                        <div class="flex flex-row items-center gap-1">
                            <UIcon name="i-mdi-account-music-outline" />
                            <span class="text-gray-400 text-xs line-clamp-1">{{ media.artist }}</span>
                        </div>
                        <div class="flex flex-row items-center gap-1">
                            <UIcon name="i-mdi-album" />
                            <span class="text-gray-400 text-xs line-clamp-1">{{ media.album }}</span>
                        </div>
                    </div>
                </div>

                <div class="lg:hidden absolute w-full h-full">
                    <div class="flex flex-col items-end justify-end w-full h-full">
                        <UDropdown :items="getMoreActions(media)" :popper="{ placement: 'bottom-start' }">
                            <UButton icon="i-mdi-dots-vertical" size="xl" variant="link" />
                        </UDropdown>
                        <div class="w-full h-full" @click="player.play(media, true)"></div>
                    </div>
                </div>
            </div>
            <div class="flex-grow"></div>
        </div>

        <div class="flex justify-end px-3 py-3.5 border-t border-gray-200 dark:border-gray-700">
            <UPagination :model-value="query.index + 1" @update:model-value="v => query.index = v - 1"
                :page-count="query.limit" :total="medias?.total ?? 0" />
        </div>
    </div>
</template>

<style scoped>
.media-overlay {
    @apply hidden absolute bg-opacity-60 bg-black rounded w-full h-full;
}

.wild-card:hover .media-overlay {
    @apply lg:block;
}
</style>