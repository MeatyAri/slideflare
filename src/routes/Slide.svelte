<script>
    let { bgColor, textColor, title, content } = $props();
</script>

<section class="w-full h-[100vh] {bgColor} {textColor}">
    <div class="flex flex-col items-center justify-center h-screen">
        <h1 class="text-4xl font-bold">{title}</h1>
        <p class="mt-4 text-lg">{content}</p>
    </div>
</section>
