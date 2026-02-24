<script>
    import Background3D from "$lib/components/Background3D.svelte";
</script>

<svelte:head>
    <title>Zen Mode - Kashiful Haque</title>
</svelte:head>

<div class="zen-container">
    <Background3D />
</div>

<style>
    .zen-container {
        position: relative;
        width: 100vw;
        height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }
</style>
