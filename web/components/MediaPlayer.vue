<script setup lang="ts">

const props = defineProps<{
    state: MediaPlayerState,
}>();

const isPlaylistOpen = ref(false);
const player = useMediaPlayer();
const pagination = reactive({
    page: 1,
    limit: 10,
})
const medias = ref<MixedMedia[]>([]);
function fetchRealMedias() {
    const index = (pagination.page - 1) * pagination.limit;
    medias.value = props.state.playlist.slice(index, index + pagination.limit);
}
watch(props.state, () => {
    if (player.total() > 0) {
        fetchRealMedias();
    } else {
        medias.value = [];
    }
});

function changeMode() {
    let mode = props.state.mode;
    if (mode < PlayMode.Shuffle) {
        mode++;
    } else {
        mode = PlayMode.Order;
    }
    if (mode !== props.state.mode) player.setMode(mode);
}

function changePlaying() {
    if (props.state.playing) {
        player.pause();
    } else {
        player.resume();
    }
}

function openList() {
    pagination.page = Math.floor(props.state.currentIndex / pagination.limit) + 1
    isPlaylistOpen.value = true;
}

function closePlayer() {
    player.stop();
    player.hide();
}
</script>

<template>
    <div v-if="state.visible">
        <div v-if="state.current"
            class="bg-white border-t border-gray-200 dark:border-gray-700 border-gray-200 dark:bg-gray-900 flex flex-col md:flex-row items-center gap-2 p-2 w-full">
            <div class="flex flex-row items-center justify-center gap-2 w-full">
                <div class="flex items-center justify-center size-16 shrink-0">
                    <img loading="lazy" class="size-full rounded object-cover" v-if="state.current.cover_url"
                        :src="setAuthQuery(state.current.cover_url)" :onerror="() => {
                            if (state.current) state.current.cover_url = undefined;
                        }">
                    <svg v-else class="text-gray-300 size-full rounded object-cover" xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24">
                        <path fill="currentColor"
                            d="M16 9h-3v5.5a2.5 2.5 0 0 1-2.5 2.5A2.5 2.5 0 0 1 8 14.5a2.5 2.5 0 0 1 2.5-2.5c.57 0 1.08.19 1.5.5V7h4zm-4-7a10 10 0 0 1 10 10a10 10 0 0 1-10 10A10 10 0 0 1 2 12A10 10 0 0 1 12 2m0 2a8 8 0 0 0-8 8a8 8 0 0 0 8 8a8 8 0 0 0 8-8a8 8 0 0 0-8-8" />
                    </svg>
                </div>
                <div class="flex flex-col gap-1 w-full">
                    <div class="flex flex-col gap-1">
                        <UTooltip :text="state.current.title">
                            <span class="font-bold text-sm line-clamp-1">{{
                                state.current.title
                            }}</span>
                        </UTooltip>
                        <div class="flex flex-row flex-wrap gap-2 items-center">
                            <div class="flex flex-row items-center gap-1">
                                <UIcon name="i-mdi-account-music-outline" />
                                <span class="text-gray-400 text-xs line-clamp-1">{{ state.current.artist }}</span>
                            </div>
                            <div class="flex flex-row items-center gap-1">
                                <UIcon name="i-mdi-album" />
                                <span class="text-gray-400 text-xs line-clamp-1">{{ state.current.album }}</span>
                            </div>
                            <div class="flex flex-row items-center gap-1">
                                <span v-if="state.current.bit_depth" class="text-gray-400 text-xs line-clamp-1">{{
                                    state.current.bit_depth }}-bit</span>
                                <UBadge variant="soft" size="xs" :label="state.current.file_type" />
                            </div>

                        </div>
                    </div>
                    <div class="flex flex-row items-center gap-2">
                        <span class="text-gray-400 text-xs">{{ toHHMMSS(state.currentElapsedSeconds) }}</span>
                        <URange class="w-full" size="sm" :max="state.current.duration_seconds"
                            :model-value="state.currentElapsedSeconds"
                            @update:model-value="secs => player.seekTo(secs)" />
                        <span class="text-gray-400 text-xs">{{ toHHMMSS(state.current.duration_seconds) }}</span>
                    </div>
                </div>
            </div>
            <div class="flex flex-row items-center justify-center gap-2">
                <UButton :disabled="!player.canBackward()" icon="i-mdi-step-backward" variant="soft" size="xl"
                    @click="player.backward()" />
                <UButton variant="soft" size="xl" @click="changePlaying">
                    <template #leading>
                        <UIcon v-if="!state.playing" name="i-mdi-play" class="size-6" />
                        <UIcon v-else name="i-mdi-pause" class="size-6" />
                    </template>
                </UButton>
                <UButton :disabled="!player.canForward()" icon="i-mdi-step-forward" variant="soft" size="xl"
                    @click="player.forward()" />
                <UButton variant="soft" @click="changeMode" size="xl">
                    <template #leading>
                        <UIcon v-if="state.mode === PlayMode.Order" name="i-mdi-sort-ascending" class="size-6" />
                        <UIcon v-else-if="state.mode === PlayMode.Repeat" name="i-mdi-repeat" class="size-6" />
                        <UIcon v-else-if="state.mode === PlayMode.RepeatOnce" name="i-mdi-repeat-once" class="size-6" />
                        <UIcon v-else name="i-mdi-shuffle-variant" class="size-6" />
                    </template>
                </UButton>
                <UButton icon="i-mdi-playlist-music" variant="soft" size="xl" @click="openList()" />
                <UButton icon="i-mdi-close" variant="soft" size="xl" @click="closePlayer()" />
            </div>
        </div>

        <UModal v-model="isPlaylistOpen">
            <UCard :ui="{
                base: 'h-full flex flex-col',
                rounded: '',
                divide: 'divide-y divide-gray-100 dark:divide-gray-800',
                body: {
                    base: 'grow'
                }
            }">
                <template #header>
                    <div class="flex items-center justify-between">
                        <h3 class="text-base font-semibold leading-6 text-gray-900 dark:text-white">
                            Playlist
                        </h3>
                        <UButton color="gray" variant="ghost" icon="i-heroicons-x-mark-20-solid" class="-my-1"
                            @click="isPlaylistOpen = false" />
                    </div>
                </template>
                <div class="flex flex-col justify-center gap-2">
                    <div class="flex flex-row items-center gap-2 flex-wrap">
                        <UPagination v-model="pagination.page" :page-count="pagination.limit"
                        @update:model-value="fetchRealMedias()" :total="player.total() ?? 0" />
                        <UBadge :label="`Total: ${player.total() ?? 0}`" variant="soft"/>
                    </div>
                    <div class="flex flex-col justify-center gap-2" v-for="media in medias">
                        <div class="flex flex-row items-center justify-between gap-2">
                            <div :class="state.current?.id === media.id ? `text-primary` : ``"
                                @click="player.skipTo(media)">
                                <div
                                    class="flex flex-col justify-center gap-2 flex-1 w-full hover:text-primary-400 hover:cursor-pointer">
                                    <div class="flex flex-col gap-1">
                                        <UTooltip :text="media.title">
                                            <span class="font-bold text-sm line-clamp-1">{{
                                                media.title
                                                }}</span>
                                        </UTooltip>
                                        <div class="flex flex-row flex-wrap gap-2 items-center  ">
                                            <div v-if="state.current?.id === media.id"
                                                class="flex flex-row items-center gap-1">
                                                <UIcon v-if="state.playing" name="i-mdi-music animate-bounce" />
                                                <UIcon v-else name="i-mdi-pause" />
                                            </div>
                                            <UIcon v-else-if="state.playedMediaIds.has(media.id)"
                                                name="i-mdi-playlist-check" />

                                            <div class="flex flex-row items-center gap-1">
                                                <UIcon name="i-mdi-account-music-outline" />
                                                <span class="text-gray-400 text-xs line-clamp-1">{{ media.artist
                                                    }}</span>
                                            </div>
                                            <div class="flex flex-row items-center gap-1">
                                                <UIcon name="i-mdi-album" />
                                                <span class="text-gray-400 text-xs line-clamp-1">{{ media.album
                                                    }}</span>
                                            </div>
                                            <div class="flex flex-row items-center gap-1">
                                                <span v-if="media.bit_depth"
                                                    class="text-gray-400 text-xs line-clamp-1">{{
                                                        media.bit_depth }}-bit</span>
                                                <UBadge variant="soft" size="xs" :label="media.file_type" />
                                            </div>
                                            <span class="text-gray-400 text-xs">{{
                                                toHHMMSS(media.duration_seconds) }}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <UButton icon="i-mdi-close" variant="link" @click="player.remove(media)" />
                        </div>
                        <UDivider />
                    </div>
                </div>
            </UCard>
        </UModal>
    </div>
</template>