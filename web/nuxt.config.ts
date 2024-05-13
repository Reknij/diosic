// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  devtools: { enabled: true },
  modules: ["@nuxt/ui"],
  ui: {
    icons: ['mdi']
  },
  app: {
    head: {
      title: 'Diosic',
      link: [
        { rel: 'icon', type: 'image/svg', href: '/favicon.svg' }
      ]
    }
  },
  ssr: false,
  nitro: {
    preset: 'static',
    devProxy: {
      "/api": {
        target: "http://0.0.0.0:3177/api",
      },
    },
  },
})