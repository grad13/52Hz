import { addMessages, init, locale } from "svelte-i18n";
import { loadLocale } from "./settings-store";
import en from "../locales/en.json";
import ja from "../locales/ja.json";

addMessages("en", en);
addMessages("ja", ja);

init({
  fallbackLocale: "en",
  initialLocale: "en",
});

// Load saved locale preference and apply (non-blocking)
export function applySavedLocale(): void {
  loadLocale()
    .then((saved) => {
      if (saved) locale.set(saved);
    })
    .catch(() => {});
}
