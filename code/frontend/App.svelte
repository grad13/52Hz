<script lang="ts">
  import "./app.css";
  import TrayPanel from "./components/TrayPanel.svelte";
  import BreakOverlay from "./components/BreakOverlay.svelte";

  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");
</script>

{#if view === "break"}
  <BreakOverlay />
{:else}
  <TrayPanel />
{/if}
