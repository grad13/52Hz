// meta: updated=2026-03-16 07:20 checked=-
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

// Load saved locale preference and apply.
// Returns a promise so callers can await before mounting UI.
export async function applySavedLocale(): Promise<void> {
  try {
    const saved = await loadLocale();
    if (saved) locale.set(saved);
  } catch {
    // ignore — falls back to "en"
  }
}
