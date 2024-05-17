<script setup lang="ts">
import type { GetSourcesQuery, ListSlice, SourceInfo } from '~/api/model';
import { getSources } from '~/api/source';

const router = useRouter();
const route = useRoute()
const source = route.params.source as string;

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

</script>

<template>
    <div class="flex flex-col gap-2 p-2 h-full">
        <div class="flex flex-row flex-wrap items-center justify-center gap-2">
            <div class="wild-card wild-card-btn p-2" @click="$router.push(`/m/${source}?f=${s.title}`)"
                v-for="s in sources?.items">
                <div class="flex flex-col items-center justify-center w-32 gap-2">
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