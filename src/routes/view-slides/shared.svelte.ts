interface Slide {
	title: string;
	content: string;
	bg_color: string;
	text_color: string;
}

interface SharedState {
	index: number;
	slides: Slide[];
}

export const shared: SharedState = $state({
	index: 0,
	slides: JSON.parse(localStorage.getItem('slides') || '[]') as Slide[] // Load from localStorage
});
