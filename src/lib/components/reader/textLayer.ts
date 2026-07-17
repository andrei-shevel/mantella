import type { TextRun } from "../../api/types";

export interface LayerRun extends TextRun {
  /** Extra selectable area around the glyphs (PDF points). */
  padTop: number;
  padBottom: number;
  padLeft: number;
  padRight: number;
}

interface Line {
  top: number;
  bottom: number;
  runs: LayerRun[];
}

/**
 * Extend each run's selection hit area so the runs tile the entire page with
 * no dead zones. Bare runs only cover their glyphs' tight rects; a selection
 * started in uncovered whitespace snaps to whatever position WebKit guesses
 * is nearest, which is often a different paragraph. Instead, every gap splits
 * at its midpoint — vertically between lines, horizontally between runs on a
 * line — and the outermost runs extend to the page edges, so a click lands on
 * the run the user is actually nearest to, like in a text editor. Only the
 * hit areas grow; the glyphs don't move.
 */
export function withHitPadding(
  runs: TextRun[],
  pageWidth: number,
  pageHeight: number,
): LayerRun[] {
  const result: LayerRun[] = runs.map((run) => ({
    ...run,
    padTop: 0,
    padBottom: 0,
    padLeft: 0,
    padRight: 0,
  }));

  // Group vertically overlapping runs into lines (multi-column layouts with
  // staggered baselines degrade gracefully: gaps come out ~0, so no padding).
  const byTop = [...result].sort((a, b) => a.y - b.y);
  const lines: Line[] = [];
  for (const run of byTop) {
    const line = lines[lines.length - 1];
    if (line && run.y + run.height / 2 < line.bottom) {
      line.top = Math.min(line.top, run.y);
      line.bottom = Math.max(line.bottom, run.y + run.height);
      line.runs.push(run);
    } else {
      lines.push({ top: run.y, bottom: run.y + run.height, runs: [run] });
    }
  }

  // Vertical tiling: every run in a line covers the full line band, each
  // inter-line gap splits at its midpoint, and the first/last lines extend
  // into the page margins.
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const above =
      i > 0 ? Math.max(0, line.top - lines[i - 1].bottom) / 2 : line.top;
    const below =
      i < lines.length - 1
        ? Math.max(0, lines[i + 1].top - line.bottom) / 2
        : Math.max(0, pageHeight - line.bottom);
    for (const run of line.runs) {
      run.padTop = run.y - line.top + above;
      run.padBottom = line.bottom - (run.y + run.height) + below;
    }
  }

  // Horizontal tiling within each line's band: split gaps between runs (word
  // groups, column gutters) at their midpoints, and extend the outermost runs
  // to the page edges so short lines own their trailing whitespace.
  for (const line of lines) {
    const byLeft = [...line.runs].sort((a, b) => a.x - b.x);
    for (let i = 0; i + 1 < byLeft.length; i++) {
      const gap = byLeft[i + 1].x - (byLeft[i].x + byLeft[i].width);
      if (gap > 0) {
        byLeft[i].padRight = gap / 2;
        byLeft[i + 1].padLeft = gap / 2;
      }
    }
    const first = byLeft[0];
    let last = byLeft[0];
    for (const run of byLeft) {
      if (run.x + run.width > last.x + last.width) last = run;
    }
    first.padLeft = Math.max(first.padLeft, first.x);
    last.padRight = Math.max(last.padRight, pageWidth - (last.x + last.width));
  }

  return result;
}
