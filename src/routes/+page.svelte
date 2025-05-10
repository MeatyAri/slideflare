<script>
    import Slide from './Slide.svelte';
    import slides from './slides.json';
</script>

<main class="flex flex-col items-center justify-center">
    {#each slides as slide}
        <Slide
            bgColor={slide.bgColor}
            textColor={slide.textColor}
            title={slide.title}
            content={slide.content}
        />
    {/each}
</main>
