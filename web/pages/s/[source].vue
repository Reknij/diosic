<script setup lang="ts">
import { PlayMode } from '#imports';
import { getMedias } from '~/api/media';
import type { GetSourcesQuery, ListSlice, SourceInfo } from '~/api/model';
import { getSources } from '~/api/source';

const router = useRouter();
const route = useRoute()
const source = route.params.source as string;
const player = useMediaPlayer();
const toast = useToast();
const query = reactive<GetSourcesQuery>({
    index: 0,
    limit: 40,
    source: parseSource(source),
})
const sourceValid = [
    "library", "category", "album", "artist", "genre", "year"
]
if (!sourceValid.find((v) => v === source)) {
    console.warn("Fetch by source is invalid!");
    router.replace('/');
}
const { data: sources } = await getSources(query);

async function playAll(mode: PlayMode, source: SourceInfo) {
    const { data, error } = await getMedias({
        index: 0,
        limit: 99999999,
        source: query.source,
        filter: source.title,
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

function getMoreActions(source: SourceInfo) {
    return [
        [
            {
                icon: 'i-mdi-playlist-play',
                label: 'Play all',
                click() {
                    playAll(PlayMode.Order, source)
                }
            },
            {
                icon: 'i-mdi-repeat',
                label: 'Repeat',
                click() {
                    playAll(PlayMode.Repeat, source)
                }
            },
            {
                icon: 'i-mdi-shuffle-variant',
                label: 'Shuffle',
                click() {
                    playAll(PlayMode.Shuffle, source)
                }
            },
            {
                icon: 'i-mdi-playlist-plus',
                label: 'Add to playlist',
                async click() {
                    const { data, error } = await getMedias({
                        index: 0,
                        limit: 99999999,
                        source: query.source,
                        filter: source.title,
                    })
                    if (error.value) {
                        toast.add({
                            color: 'red',
                            title: error.value.data
                        })
                    } else if (data.value) {
                        player.pushList(data.value.items);
                        player.show();
                    }
                }
            },
            {
                icon: 'i-mdi-playlist-minus',
                label: 'Remove from playlist',
                async click() {
                    const { data, error } = await getMedias({
                        index: 0,
                        limit: 99999999,
                        source: query.source,
                        filter: source.title,
                    })
                    if (error.value) {
                        toast.add({
                            color: 'red',
                            title: error.value.data
                        })
                    } else if (data.value) {
                        player.removeList(data.value.items);
                        player.show();
                    }
                }
            },
        ]
    ]
}

</script>

<template>
    <div class="flex flex-col gap-2 p-2 h-full">
        <div class="flex flex-row flex-wrap items-center justify-center gap-2">
            <div class="wild-card wild-card-btn p-2" v-for="s in sources?.items">
                <div class="flex flex-row items-center justify-end gap-0">
                    <UDropdown :items="getMoreActions(s)" :popper="{ placement: 'bottom-start' }">
                        <UButton icon="i-mdi-dots-vertical" size="xl" variant="link" :padded="false" />
                    </UDropdown>
                </div>
                <div class="flex flex-col items-center justify-center w-32 gap-2"
                    @click="$router.push(`/m/${source}?f=${s.title}`)">
                    <span class="font-bold line-clamp-1">{{ s.title }}</span>
                    <UBadge class="w-fit" variant="subtle" :label="`Total: ${s.total_media}`" />
                </div>
            </div>
        </div>
        <div class="flex-grow"></div>
        <div class="flex justify-end px-3 py-3.5 border-t border-gray-200 dark:border-gray-700">
            <UPagination :model-value="query.index + 1" @update:model-value="v => query.index = v - 1"
                :page-count="query.limit" :total="sources?.total ?? 0" />
        </div>
    </div>
</template>