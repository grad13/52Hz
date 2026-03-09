// meta: ref=App
import "./lib/i18n";
import App from "./App.svelte";
import { mount } from "svelte";
import { applySavedLocale } from "./lib/i18n";

const app = mount(App, {
  target: document.getElementById("app")!,
});

applySavedLocale();

export default app;
