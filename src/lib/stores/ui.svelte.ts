class UiStore {
  sidebarOpen = $state(true);

  toggleSidebar() {
    this.sidebarOpen = !this.sidebarOpen;
  }
}

export const ui = new UiStore();
