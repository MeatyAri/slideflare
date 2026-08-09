/**
 * Tutorial feature registry — single source of truth.
 *
 * Add a new feature by appending an entry. Gating is by feature `id`, not by
 * app version:
 *   - First ever launch  -> every feature is shown (full "welcome" tutorial).
 *   - Afterwards         -> only features whose `id` the user hasn't seen yet
 *                           are shown ("what's new").
 *
 * Because gating keys off `id`, a new card surfaces the moment its entry lands
 * in this list — HEAD-tracking git/AUR builds get it before any release tags a
 * new version.
 *
 * Keep `id` stable and unique forever: changing it re-shows the card to every
 * user. Reordering or rewording an entry is safe as long as its `id` stays put.
 */
export interface TutorialFeature {
  /** Stable unique key. Drives gating — never reuse, repurpose, or change it. */
  id: string;
  /**
   * App version this feature shipped in, e.g. '0.1.1'. Cosmetic only: used to
   * derive the "What's new in vX.Y.Z" header label, never for gating.
   */
  version: string;
  /** Short heading. */
  title: string;
  /** One or more sentences describing the feature. Plain text. */
  body: string;
  /**
   * Optional media (image/gif) shown above the text. Path relative to the
   * static/ dir, e.g. '/tutorial/hot-reload.gif'. When present, the feature
   * always gets its own carousel slide.
   */
  media?: string;
  /**
   * Marks the card that offers to install/update the slideflare-slides skill.
   * The overlay renders an install button on it. Only one feature should set
   * this.
   */
  skillInstall?: boolean;
}

export const TUTORIAL_FEATURES: TutorialFeature[] = [
  {
    id: 'open-file',
    version: '0.1.0',
    title: 'Open a deck',
    body: 'Drag & drop a Markdown (.md) file onto the window, or click to browse and pick one.'
  },
  {
    id: 'hot-reload',
    version: '0.1.0',
    title: 'Live reload',
    body: 'Edit your Markdown in any editor and the slides update instantly as you save.'
  },
  {
    id: 'navigate',
    version: '0.1.0',
    title: 'Navigate',
    body: 'Use the arrow keys to move between slides.'
  },
  {
    id: 'latex',
    version: '0.1.0',
    title: 'Math with LaTeX',
    body: 'Write $inline$ math and $$display$$ math directly in your Markdown.'
  },
  {
    id: 'styling',
    version: '0.1.0',
    title: 'Style with Tailwind',
    body: 'Set background and text colors per slide from YAML frontmatter, and use HTML + Tailwind classes for custom layouts.'
  },
  {
    id: 'media',
    version: '0.1.0',
    title: 'Rich media',
    body: 'Embed images (PNG, JPG, GIF, SVG, WebP) and videos (MP4, WebM, and more) with paths relative to your Markdown file.'
  },
  {
    id: 'error-recovery',
    version: '0.1.0',
    title: 'Error recovery',
    body: 'Invalid slide syntax shows a clear error with the offending line, and recovers automatically once you fix it.'
  },
  {
    id: 'ai-decks',
    version: '0.1.0',
    title: 'Generate decks with AI',
    body: 'Use the slideflare-slides skill to turn a plain-language prompt into a complete, styled Markdown deck. Install it into ~/.agents/skills/ with one click — your agent picks it up automatically.',
    skillInstall: true
  },
  {
    id: 'reload',
    version: '0.1.1',
    title: 'Force reload',
    body: 'Press Ctrl + R to reparse the whole file from scratch whenever something looks off.'
  },
  {
    id: 'exit',
    version: '0.1.1',
    title: 'Exit to home',
    body: 'Press Esc or click the back button to return to the file picker.'
  },
  {
    id: 'updates',
    version: '0.1.2',
    title: 'Check for updates',
    body: 'The button on the home screen checks for both app and skill updates in one go.'
  }
];
