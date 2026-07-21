export interface MenuItem {
  label: string;
  danger?: boolean;
  action: () => void;
}

class UiStore {
  sidebarOpen = $state(true);
  bookmarksPanelOpen = $state(false);
  settingsOpen = $state(false);
  contextMenu = $state<{ x: number; y: number; items: MenuItem[] } | null>(
    null,
  );

  toggleSidebar() {
    this.sidebarOpen = !this.sidebarOpen;
  }

  toggleBookmarksPanel() {
    this.bookmarksPanelOpen = !this.bookmarksPanelOpen;
  }

  openSettings() {
    this.settingsOpen = true;
  }

  closeSettings() {
    this.settingsOpen = false;
  }

  openContextMenu(e: MouseEvent, items: MenuItem[]) {
    e.preventDefault();
    this.contextMenu = { x: e.clientX, y: e.clientY, items };
  }

  closeContextMenu() {
    this.contextMenu = null;
  }
}

export const ui = new UiStore();
