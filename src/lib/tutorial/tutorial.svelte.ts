import { TUTORIAL_FEATURES, type TutorialFeature } from './features';

const SEEN_FEATURES_KEY = 'tutorialSeenFeatures';

/** Body length at/under which a feature is considered a short "one-liner". */
const SHORT_BODY_MAX = 90;
/** Max short one-liners packed onto a single grouped carousel slide. */
const MAX_GROUP_SIZE = 3;

/**
 * Compare two dotted version strings numerically.
 * Returns <0 if a<b, 0 if equal, >0 if a>b. Missing parts treated as 0.
 */
export function compareVersions(a: string, b: string): number {
  const pa = a.split('.').map((n) => parseInt(n, 10) || 0);
  const pb = b.split('.').map((n) => parseInt(n, 10) || 0);
  const len = Math.max(pa.length, pb.length);
  for (let i = 0; i < len; i++) {
    const diff = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (diff !== 0) return diff;
  }
  return 0;
}

/**
 * A single page of the tutorial carousel. Either:
 *   - one feature that owns the whole slide (has media, or long text), or
 *   - a group of short one-liner features shown together.
 */
export interface TutorialSlide {
  features: TutorialFeature[];
  /** True when this slide packs multiple short features into a list. */
  grouped: boolean;
}

/**
 * Which features should be shown, in registry order.
 *
 * Gating is by feature `id`, not by app version: a feature is "new" until the
 * user has dismissed a tutorial that included it. This lets HEAD-tracking git /
 * AUR builds surface a feature the moment its registry entry lands, before any
 * release tags a new version.
 */
export function featuresToShow(seen: Set<string>): TutorialFeature[] {
  return TUTORIAL_FEATURES.filter((f) => !seen.has(f.id));
}

/**
 * Pack features into carousel slides.
 *
 * Rules:
 *   - A feature with media always gets its own slide.
 *   - A feature with a long body (no media) gets its own slide.
 *   - Short one-liners (no media, body <= SHORT_BODY_MAX) are batched together,
 *     up to MAX_GROUP_SIZE per slide, so a lone sentence never wastes a slide.
 */
export function buildSlides(features: TutorialFeature[]): TutorialSlide[] {
  const slides: TutorialSlide[] = [];
  let pending: TutorialFeature[] = [];

  const flush = () => {
    if (pending.length === 0) return;
    slides.push({ features: pending, grouped: pending.length > 1 });
    pending = [];
  };

  for (const f of features) {
    const isShort = !f.media && f.body.length <= SHORT_BODY_MAX;
    if (isShort) {
      pending.push(f);
      if (pending.length >= MAX_GROUP_SIZE) flush();
    } else {
      flush();
      slides.push({ features: [f], grouped: false });
    }
  }
  flush();

  return slides;
}

export function getSeenFeatures(): Set<string> {
  try {
    const raw = localStorage.getItem(SEEN_FEATURES_KEY);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? new Set(parsed as string[]) : new Set();
  } catch {
    return new Set();
  }
}

export function markSeen(ids: string[]): void {
  const seen = getSeenFeatures();
  for (const id of ids) seen.add(id);
  localStorage.setItem(SEEN_FEATURES_KEY, JSON.stringify([...seen]));
}

/** Highest feature `version` among the given features, for header display. */
function maxVersion(features: TutorialFeature[]): string {
  return features.reduce(
    (max, f) => (compareVersions(f.version, max) > 0 ? f.version : max),
    '0.0.0'
  );
}

/**
 * Reactive tutorial controller.
 *
 * Gating is id-based (see featuresToShow). The app version is used only for the
 * "what's new" header label, derived from the newest feature being shown so it
 * stays accurate even for features that haven't been released/tagged yet.
 */
export function createTutorial() {
  const state = $state({
    open: false,
    /** 'welcome' on the very first launch, 'whatsnew' when new cards appear. */
    mode: 'welcome' as 'welcome' | 'whatsnew',
    version: '',
    slides: [] as TutorialSlide[]
  });

  let shownIds: string[] = [];

  function init(): void {
    const seen = getSeenFeatures();
    const isFirstLaunch = seen.size === 0;
    const features = featuresToShow(seen);

    if (features.length === 0) return; // Nothing new to surface.

    shownIds = features.map((f) => f.id);
    state.mode = isFirstLaunch ? 'welcome' : 'whatsnew';
    state.version = maxVersion(features);
    state.slides = buildSlides(features);
    state.open = true;
  }

  function dismiss(): void {
    state.open = false;
    markSeen(shownIds);
  }

  return {
    get open() {
      return state.open;
    },
    get mode() {
      return state.mode;
    },
    get version() {
      return state.version;
    },
    get slides() {
      return state.slides;
    },
    init,
    dismiss
  };
}
