export default defineNuxtPlugin((nuxtApp) => {
    nuxtApp.hook("vue:error", (error, instance, info) => {
      console.error(`error-handler: ${error}`);
      console.warn(`error-handler: ${instance}`);
      console.info(`error-handler: ${info}`);
    });
  });