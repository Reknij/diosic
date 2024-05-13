<script setup lang="ts">
const route = useRoute()
const source = route.params.source as string;
const filter = route.query.f as string;
</script>

<template>
    <div class="flex flex-col gap-2 p-2">
        <Medias :source="parseSource(source)" :filter="filter" />
    </div>
</template>