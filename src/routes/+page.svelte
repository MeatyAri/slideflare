<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { webview } from '@tauri-apps/api';
	import { listen } from '@tauri-apps/api/event';
	import { open } from '@tauri-apps/plugin-dialog';

	let dragActive = false;
	let markdownContent = '';
	let error = '';

	async function handleFile(filePath: String) {
		try {
			// Invoke the Rust command with the file paths
			await invoke('start_file_watcher', { filePath })
				.then(() => console.log('File watcher started'))
				.catch((error) => console.error('Failed to start file watcher:', error));
			console.log(`Watching: ${filePath}`);
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
			handleFile(selected);
		} else {
			console.log('No file selected');
		}
	}

	listen('markdown-updated', (event) => {
		const markdownContent = event.payload;
		// Update your frontend UI with the new markdown content
		console.log('Markdown content updated:', markdownContent);
	});
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
			<svg
				class="h-12 w-12 text-blue-400"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M7 16V4a1 1 0 011-1h8a1 1 0 011 1v12m-5 4v-4m0 0l-3 3m3-3l3 3"
				/>
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

	{#if markdownContent}
		<div class="mt-8 w-full max-w-lg rounded bg-gray-800 p-4 shadow">
			<h2 class="mb-2 text-lg font-bold text-gray-100">Markdown Content</h2>
			<pre class="whitespace-pre-wrap text-gray-300">{markdownContent}</pre>
		</div>
	{/if}
</div>
