/**
 * Fixed virtual design resolution. Every slide is laid out at exactly these
 * dimensions and then uniformly scaled to fit the real window, so the layout is
 * pixel-identical at any screen resolution or window size (16:9).
 */
export const DESIGN_W = 1280;
export const DESIGN_H = 720;

/** Tiny padding (px) left around the scaled slide canvas. */
export const VIEWPORT_PADDING = 24;

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
  /**
   * While true the deck lays itself out for paper instead of for the window:
   * every slide is scaled against the fixed design resolution rather than the
   * viewport. The PDF export sets this before handing the live webview to the
   * platform print pipeline, and clears it once the export reports back.
   */
  printMode: boolean;
  /**
   * Counts parses delivered by *this* process — incremented on every
   * `markdown-updated` and `slide-changed` event.
   *
   * `slides` starting empty already means a non-empty deck can only have come
   * from this process, so this is a second, cheaper way to say the same thing.
   * It stays because it also distinguishes "parsed, and legitimately empty"
   * from "nothing has arrived yet".
   */
  generation: number;
}

export type { ParseError, Slide };

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
  // Deliberately empty, never seeded from localStorage. Restoring the previous
  // deck here bought a slightly smoother reload at the cost of making "there are
  // slides" meaningless: at startup those slides belong to whatever file was
  // open last. Anything waiting for a deck to be ready — CLI export above all,
  // which would otherwise render the wrong document — could not tell the
  // difference. Rust re-parses and re-sends on every launch anyway, so nothing
  // is lost by starting blank.
  slides: [],
  error: null,
  printMode: false,
  generation: 0
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
}
