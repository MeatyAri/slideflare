export const shared = $state({
    index: 0,
    slides: JSON.parse(localStorage.getItem('slides') || '[]'), // Load from localStorage
});
