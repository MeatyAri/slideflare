<script>
    import NavBar from './NavBar.svelte';
    import Slide from './Slide.svelte';
    import slides from './slides.json';
</script>

<NavBar />

<main class="flex flex-col items-center justify-center">
    {#each slides as slide, index}
        <Slide
            id={index}
            bgColor={slide.bgColor}
            textColor={slide.textColor}
            title={slide.title}
            content={slide.content}
        />
    {/each}
</main>
