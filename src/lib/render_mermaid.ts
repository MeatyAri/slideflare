import mermaid from 'mermaid';

mermaid.initialize({
	startOnLoad: false
	// theme: 'default',
	// // Make diagrams responsive and constrain their size
	// maxTextSize: 50000,
	// // Use CSS for better control
	// logLevel: 4
});

export async function renderMermaidDiagrams() {
	const mermaidElements = document.querySelectorAll('.mermaid');
	for (const element of mermaidElements) {
		const graphDefinition = element.textContent || '';
		try {
			const { svg, bindFunctions } = await mermaid.render(
				`mermaid-${Math.random().toString(36).substring(2, 11)}`,
				graphDefinition
			);
			element.innerHTML = svg;
			if (bindFunctions) bindFunctions(element);

			// Apply responsive styling to the rendered SVG
			const svgElement = element.querySelector('svg');
			if (svgElement) {
				svgElement.classList.add('mermaid-diagram');
				// Calculate and set the maximum height for the SVG container
				const maxHeight = window.getComputedStyle(element).height;
				console.log(maxHeight);
				if (maxHeight && maxHeight !== 'none') {
					svgElement.style.maxHeight = `${parseFloat(maxHeight) * 0.8}px`;
				}
			}
		} catch (error) {
			console.error('Mermaid rendering error:', error);
			// @ts-ignore
			element.innerHTML = `<pre>Error rendering diagram: ${error.message}</pre>`;
		}
	}
}
