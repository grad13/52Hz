<script lang="ts">
  import "./app.css";
  import Settings from "./components/Settings.svelte";
  import BreakOverlay from "./components/BreakOverlay.svelte";

  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");
</script>

{#if view === "break"}
  <BreakOverlay />
{:else}
  <Settings />
{/if}
