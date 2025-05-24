<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { webview } from '@tauri-apps/api';
	import { open } from '@tauri-apps/plugin-dialog';
	import { goto } from '$app/navigation';

	let dragActive = $state(false);
	let error = $state('');

	async function handleFile(filePath: String) {
		try {
			// Invoke the Rust command with the file paths
			await invoke('start_file_watcher', { filePath }).catch((error) =>
				console.error('Failed to start file watcher:', error)
			);
		} catch (error) {
			console.error('Error processing files:', error);
		}
	}

	function onDrop(event: DragEvent) {
		event.preventDefault();
		dragActive = false;
	}

	function onDragOver(event: DragEvent) {
		event.preventDefault();
		dragActive = true;
	}

	function onDragLeave(event: DragEvent) {
		event.preventDefault();
		dragActive = false;
	}

	webview.getCurrentWebview().onDragDropEvent(async (event) => {
		if (event.payload.type === 'drop') {
			const filePaths = event.payload.paths;
			if (!filePaths || filePaths.length === 0) return;
			const filePath = filePaths[0];
			if (!filePath.endsWith('.md')) {
				error = 'Please select a Markdown (.md) file.';
				return;
			}
			error = '';

			handleFile(filePath);
		}
	});

	async function selectFile() {
		error = '';

		const selected = await open({
			multiple: false,
			filters: [
				{
					name: 'Markdown Files',
					extensions: ['md']
				}
			]
		});

		if (typeof selected === 'string') {
			await goto('/view-slides');
			handleFile(selected);
		} else {
			console.log('No file selected');
		}
	}
</script>

<div
	class="flex min-h-screen flex-col items-center justify-center bg-gray-900 text-gray-100 select-none"
>
	<div
		class={`w-full max-w-lg cursor-pointer rounded-lg border-2 border-dashed p-8 transition-colors
            ${dragActive ? 'border-blue-500 bg-blue-800' : 'border-gray-700 bg-gray-800'}`}
		ondrop={onDrop}
		ondragover={onDragOver}
		ondragleave={onDragLeave}
		onclick={selectFile}
		aria-label="Drag and drop a Markdown file here or click to select"
		role="button"
		tabindex="0"
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				selectFile();
			}
		}}
	>
		<div class="flex flex-col items-center space-y-2">
			<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24">
				<g fill="none" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" d="M9 13h6M9 9h4m-4 8h4" /><path
						d="M19 13v2c0 2.828 0 4.243-.879 5.121C17.243 21 15.828 21 13 21h-2c-2.828 0-4.243 0-5.121-.879C5 19.243 5 17.828 5 15V9c0-2.828 0-4.243.879-5.121C6.757 3 8.172 3 11 3"
					/>
					<path stroke-linecap="round" d="M18 3v6m3-3h-6" />
				</g>
			</svg>
			<p class="font-semibold text-gray-300">Drag & drop a Markdown file here</p>
			<p class="text-sm text-gray-400">
				or <span class="cursor-pointer text-blue-400 underline">browse</span> to select
			</p>
		</div>
		{#if error}
			<p class="mt-4 text-center text-sm text-red-400">{error}</p>
		{/if}
	</div>

	<!-- <button class="p-4 bg-amber-400 cursor-pointer" onclick={() => {
        webview.getCurrentWebview().emit('terminate-event');
    }}>terminate</button> -->

	<!-- {#if markdownContent}
		<div class="mt-8 w-full max-w-lg rounded bg-gray-800 p-4 shadow">
			<h2 class="mb-2 text-lg font-bold text-gray-100">Markdown Content</h2>
			<article class="dark prose lg:prose-xl">{@html markdownContent}</article>
		</div>
	{/if} -->
</div>
