interface Slide {
	title: string;
	content: string;
	bgColor: string;
	textColor: string;
}

interface SharedState {
	index: number;
	slides: Slide[];
}

export const shared: SharedState = $state({
	index: 0,
	slides: JSON.parse(localStorage.getItem('slides') || '[]') as Slide[] // Load from localStorage
});
