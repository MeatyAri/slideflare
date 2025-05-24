<script>
    // import EmbeddedContent from "./EmbeddedContent.svelte";
	let { id, bgColor, textColor, title, content } = $props();

    // let previewFrame = $state();
    // let previewContent = EmbeddedContent;
    // previewContent.arguments = { content };

    // function updatePreview() {
    //     const doc = previewFrame.contentDocument;
    //     doc.open();
    //     doc.write(previewContent);
    //     doc.close();
    // }
</script>

<section {id} class="h-screen w-full {bgColor}">
	<div class="h-screen w-full px-24">
		<div class="flex h-screen flex-col items-center justify-center">
            <!-- <iframe bind:this={previewFrame} title="Markdown Preview" class="h-full w-full border-0"></iframe> -->
			<article class="prose lg:prose-xl dark:prose-invert {textColor}">{@html content}</article>
		</div>
	</div>
</section>
