<script setup lang="ts">
import '~/styles/global.css'

useHead({
  titleTemplate: (title) => {
    return title !== 'Diosic'
      ? `${title} - Diosic`
      : ''
  }
});

</script>

<template>
  <div>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
    <UNotifications />
    <UModals />
  </div>
</template>