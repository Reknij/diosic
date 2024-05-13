<script setup lang="ts">
import { Source } from '~/api/model';

</script>

<template>
    <div class="flex flex-col gap-2 p-2">
        <Medias :source="Source.Any" />
    </div>
</template>