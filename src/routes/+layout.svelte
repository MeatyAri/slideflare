<script>
	import '../app.css';
    import { onMount } from 'svelte';

	let { children } = $props();

    onMount(() => {
        // Prevent default scroll on wheel and touch
        // @ts-ignore
        const preventScroll = (e) => {
            e.preventDefault();
        };
        window.addEventListener('wheel', preventScroll, { passive: false });
        window.addEventListener('touchmove', preventScroll, { passive: false });
        window.addEventListener('keydown', preventScroll, { passive: false });

        return () => {
            window.removeEventListener('wheel', preventScroll);
            window.removeEventListener('touchmove', preventScroll);
            window.removeEventListener('keydown', preventScroll);
        };
    });
</script>

{@render children()}
