import { invoke } from '@tauri-apps/api/core';

const INSTALL_SOURCE_KEY = 'installSource';

/**
 * How SlideFlare was installed. Normally baked in at build time; when the build
 * didn't record it we ask the user once and remember the answer here.
 */
export type InstallSource =
  | 'github'
  | 'aur'
  | 'aur-git'
  | 'npm'
  | 'bun'
  | 'cargo'
  | 'source'
  | 'unknown';

/** Options offered when the install source has to be picked by hand. */
export const INSTALL_SOURCE_CHOICES: { value: InstallSource; label: string; hint: string }[] = [
  { value: 'github', label: 'GitHub release', hint: 'Downloaded a binary from the releases page' },
  { value: 'aur', label: 'AUR — slideflare', hint: 'Arch stable package' },
  {
    value: 'aur-git',
    label: 'AUR — slideflare-git',
    hint: 'Arch package tracking the latest commit'
  },
  { value: 'npm', label: 'npm', hint: 'Installed globally with npm' },
  { value: 'bun', label: 'bun', hint: 'Installed globally with bun' },
  { value: 'cargo', label: 'cargo', hint: 'Installed with cargo install' },
  { value: 'source', label: 'Built from source', hint: 'Cloned the repo and built it yourself' }
];

/** Shape returned by the `check_updates` Rust command. */
export interface UpdateStatus {
  installSource: InstallSource;
  /** True when no source was recorded at build time — the UI must ask. */
  installSourceUnknown: boolean;
  /** Running build: semver, or short commit for git installs. */
  current: string;
  /** Latest upstream: release version, or short commit for git installs. */
  latest: string | null;
  appUpdateAvailable: boolean;
  /** Command that updates this install, when one can be named. */
  updateCommand: string | null;
  releaseUrl: string | null;
  /** Set when the check failed (offline, rate limited, ...). */
  error: string | null;
  skillInstalled: boolean;
}

export function getStoredInstallSource(): InstallSource | null {
  try {
    const raw = localStorage.getItem(INSTALL_SOURCE_KEY);
    return raw ? (raw as InstallSource) : null;
  } catch {
    return null;
  }
}

export function setStoredInstallSource(source: InstallSource): void {
  try {
    localStorage.setItem(INSTALL_SOURCE_KEY, source);
  } catch {
    // Storage unavailable: the user just gets asked again next time.
  }
}

/**
 * Reactive controller for the update dialog.
 *
 * Checks the app and the skill together, and handles the case where the install
 * source is unknown by surfacing a picker before it can report an app update.
 */
export function createUpdates() {
  const state = $state({
    open: false,
    checking: false,
    status: null as UpdateStatus | null,
    /** Set while the skill is being installed/updated. */
    installingSkill: false,
    skillMessage: '',
    skillError: ''
  });

  async function check(): Promise<void> {
    state.checking = true;
    state.skillMessage = '';
    state.skillError = '';
    try {
      state.status = await invoke<UpdateStatus>('check_updates', {
        userInstallSource: getStoredInstallSource()
      });
    } catch (e) {
      state.status = {
        installSource: 'unknown',
        installSourceUnknown: false,
        current: '',
        latest: null,
        appUpdateAvailable: false,
        updateCommand: null,
        releaseUrl: null,
        error: String(e),
        skillInstalled: false
      };
    } finally {
      state.checking = false;
    }
  }

  function open(): void {
    state.open = true;
    check();
  }

  function close(): void {
    state.open = false;
  }

  /** Record the user's install source, then re-check with it applied. */
  async function chooseInstallSource(source: InstallSource): Promise<void> {
    setStoredInstallSource(source);
    await check();
  }

  async function installSkill(): Promise<void> {
    state.installingSkill = true;
    state.skillMessage = '';
    state.skillError = '';
    try {
      state.skillMessage = await invoke<string>('install_skill', {
        userInstallSource: getStoredInstallSource()
      });
      if (state.status) state.status.skillInstalled = true;
    } catch (e) {
      state.skillError = String(e);
    } finally {
      state.installingSkill = false;
    }
  }

  return {
    get open() {
      return state.open;
    },
    get checking() {
      return state.checking;
    },
    get status() {
      return state.status;
    },
    get installingSkill() {
      return state.installingSkill;
    },
    get skillMessage() {
      return state.skillMessage;
    },
    get skillError() {
      return state.skillError;
    },
    openDialog: open,
    close,
    check,
    chooseInstallSource,
    installSkill
  };
}
