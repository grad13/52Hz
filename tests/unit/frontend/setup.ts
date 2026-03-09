/**
 * Vitest setup for frontend tests.
 *
 * Initializes svelte-i18n with the English locale so that $_() calls
 * in components resolve to English strings during testing.
 */
import { addMessages, init } from "svelte-i18n";
import en from "@code/frontend/locales/en.json";

addMessages("en", en);
init({ fallbackLocale: "en", initialLocale: "en" });
