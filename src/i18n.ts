import { createI18n } from "vue-i18n";
import zh from "./locales/zh.json";
import en from "./locales/en.json";
import zhTW from "./locales/zh-TW.json";

const savedLocale = localStorage.getItem("vab-locale") || "zh";

const i18n = createI18n({
  legacy: false,
  locale: savedLocale,
  fallbackLocale: "en",
  messages: {
    zh,
    en,
    "zh-TW": zhTW,
  },
});

export default i18n;
