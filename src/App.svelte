<script lang="ts">
  import "./app.css";
  import Settings from "./lib/Settings.svelte";
  import BreakOverlay from "./lib/BreakOverlay.svelte";

  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");
</script>

{#if view === "break"}
  <BreakOverlay />
{:else}
  <Settings />
{/if}
