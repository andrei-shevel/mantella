# Changelog

All notable changes to Mantella are documented here. Each version's section is
used verbatim as the GitHub Release notes and as the `notes` field in
`latest.json` shown by the in-app updater — see "Publish a release" in the
README.

## [0.1.1] - 2026-07-24

### Added

- Right-click a file or folder in the sidebar to reveal it in Finder (macOS) / Explorer (Windows) / file manager.
- Full keyboard navigation for the file list and bookmarks panel: arrow keys to move, Enter/Space to open, ←/→ to expand/collapse folders, Home/End to jump to the ends, Esc to return to the document.

### Fixed

- ⌘← and ⌘→ now behave predictably: opening a closed panel focuses it, and toggling a focused panel closed hands focus back to the document instead of leaving it stranded.
- The focus ring on sidebar/bookmark rows no longer lit up on mouse clicks — it's now driven by the keyboard cursor instead of native `:focus-visible`.
- Right-clicking empty space no longer surfaces the native WebView context menu (Reload/Inspect Element).

## [0.1.0] - 2026-07-21

### Added

- PDF library with folder scanning, live sync, search, and pins.
- Crisp pdfium rendering with text selection and clickable links.
- Reading position, navigation history, and bookmarks saved per file.
- Customizable keyboard shortcuts.
- Auto-updates via signed, notarized GitHub releases.
- macOS, Windows, and Linux builds.
