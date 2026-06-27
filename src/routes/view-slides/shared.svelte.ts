interface Slide {
  title: string;
  content: string;
  bg_color: string;
  text_color: string;
}

interface SlideChangeType {
  Added?: { index: number; slide: Slide };
  Modified?: { index: number; old_hash: number; new_hash: number; slide: Slide };
  Removed?: { index: number; old_hash: number };
}

interface SlideChangeEvent {
  changes: SlideChangeType[];
}

interface ParseError {
  message: string;
  line: number | null;
}

interface SharedState {
  index: number;
  slides: Slide[];
  error: ParseError | null;
}

export type { ParseError };

export type NotificationColor = 'blue' | 'red' | 'yellow';

interface Notification {
  id: number;
  message: string;
  color: NotificationColor;
}

export const notifications: Notification[] = $state([]);

let nextId = 0;

export function notify(message: string, color: NotificationColor = 'blue'): void {
  notifications.push({ id: nextId++, message, color });
}

export function dismissNotification(id: number): void {
  const index = notifications.findIndex((n) => n.id === id);
  if (index !== -1) {
    notifications.splice(index, 1);
  }
}

export const shared: SharedState = $state({
  index: 0,
  slides: JSON.parse(localStorage.getItem('slides') || '[]') as Slide[],
  error: null
});

export function applySlideChange(event: SlideChangeEvent): void {
  event.changes.forEach((change) => {
    if (change.Removed) {
      // Remove slide at index
      const { index } = change.Removed;
      shared.slides = [...shared.slides.slice(0, index), ...shared.slides.slice(index + 1)];
    } else if (change.Added) {
      // Insert new slide at specific index
      const { index, slide } = change.Added;
      shared.slides = [...shared.slides.slice(0, index), slide, ...shared.slides.slice(index)];
    }
  });

  // Save to localStorage
  localStorage.setItem('slides', JSON.stringify(shared.slides));
}
