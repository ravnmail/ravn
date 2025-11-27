import tailwindcss from "@tailwindcss/vite";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    '@nuxt/eslint',
    "@vueuse/nuxt",
    "nuxt-i18n-micro"
  ],

  experimental: {
    watcher: "parcel",
  },

  ssr: false,
  compatibilityDate: "2025-08-24",
  devServer: { host: process.env.TAURI_DEV_HOST || "localhost" },
  devtools: { enabled: false },

  css: ["~/assets/css/main.css"],

  i18n: {
    strategy: "no_prefix",
    defaultLocale: "en",
    fallbackLocale: "en",
    translationDir: "i18n",
    define: false,
    disablePageLocales: true,
    locales: [{ code: "en", name: "English", iso: "en", file: "en.json" }],
  },

  // Enhanced Nitro configuration
  nitro: {
  },

  app: {
    head: {
      bodyAttrs: {
        class: 'dark',
      },
    },
  },

  // Enhanced Vite configuration
  vite: {
    clearScreen: false,
    envPrefix: ["VITE_", "TAURI_"],
    plugins: [tailwindcss()],
    server: {
      fs: {
        strict: false,
        allow: ['..'] // Allow serving files from one level up
      },
      strictPort: true,
    },
    // Add build optimizations
    build: {
      target:
        process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari13",
      sourcemap: !!process.env.TAURI_DEBUG,
    },
    // Add optimizations for dev server
    optimizeDeps: {
      exclude: ["fsevents", "@tauri-apps/api", "@tauri-apps/cli"],
    },
  },
});
