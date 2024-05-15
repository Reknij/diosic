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
  runtimeConfig: {
    public: {
      baseUrl: '',
      devBaseUrl: 'http://127.0.0.1:3177'
    }
  },
  nitro: {
    preset: 'static',
  },
})