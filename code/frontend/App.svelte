<script lang="ts">
  import "./app.css";
  import TrayPanel from "./components/TrayPanel.svelte";
  import BreakOverlay from "./components/BreakOverlay.svelte";
  import FocusDonePopup from "./components/FocusDonePopup.svelte";
  import Toast from "./components/Toast.svelte";

  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");

  if (view === "toast" || view === "focus-done") {
    document.body.style.background = "transparent";
  }
</script>

{#if view === "break"}
  <BreakOverlay />
{:else if view === "focus-done"}
  <FocusDonePopup />
{:else if view === "toast"}
  <Toast />
{:else}
  <TrayPanel />
{/if}
