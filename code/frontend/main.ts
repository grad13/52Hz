// meta: ref=App
import "./lib/i18n";
import App from "./App.svelte";
import { mount } from "svelte";
import { applySavedLocale } from "./lib/i18n";

// Apply saved locale BEFORE mounting so short-lived windows
// (FocusDonePopup, Toast) render in the correct language.
applySavedLocale().then(() => {
  mount(App, {
    target: document.getElementById("app")!,
  });
});

