## Overlays and the window titlebar

The app runs with `decorations: false`, so the shell renders its own titlebar.
Vainlla shadcn-svelte components assume that a modal owns the whole viewport, so the
overlay wrappers under `src/lib/components/ui/` now carry a small, deliberate set of
changes to modify this behaviour and accomodate the titlebar.

## Look out for

**These files are vendored source.** `shadcn-svelte add` copies over them, so
adding or re-adding a component silently drops everything below. Nothing errors —
the titlebar just stops behaving. Re-apply and re-check after any `add`.

**A new modal with a backdrop needs all three changes.** Drawer or any generated
`*Dialog`: `z-backdrop` on the overlay; `z-overlay-content` + `--overlay-inset`
geometry on the content; and `onInteractOutside` wrapped through
`handleWindowControlsInteractOutside`. A floating popup (ContextMenu, Menubar,
HoverCard, Combobox, NavigationMenu) only needs `z-50` → `z-floating` — skipping
that is cosmetic, it just paints under an open dialog.

**Tab cannot reach the titlebar while an overlay is open, this is deliberate.**
Modals keep bits-ui's focus trap, so focus cycles inside them. Making the
titlebar keyboard-reachable means giving up the trap, which then needs another
way to define what stays reachable, which means writing custom DOM javascript
state handling - not worth it.

**`trapFocus` is not the only modal side effect.** bits-ui also sets
`pointer-events: none` on `<body>` while a modal is open, which is why the
titlebar needs `pointer-events-auto` — without it the controls stay visible but
stop receiving clicks.

**Portaled overlays rely on z-index, not DOM order.** Body children with equal
`z-index` fall back to DOM order, and portaled overlays are always later. Never
"simplify" the layer tokens back to a shared `z-50`.

**A passed `class` overrides the wrapper's positioning.** `onInteractOutside` is
forwarded *after* the exemption, so `(e) => e.preventDefault()` still blocks
backdrop dismissal. Import from `$lib/components/ui/*`, not `bits-ui` directly —
the primitives have none of this.

## What was done and why

1. **Layer scale** (`src/app.css`) — because overlays, menus and tooltips portal
   to `<body>` and so paint after the shell:

   `backdrop (40) < titlebar (50) < menus (60) < overlay content (70) < menus opened from inside an overlay (80)`

2. **Overlay geometry** (`src/app.css`, `routes/+layout.svelte`) — backdrops and
   panels offset by `--overlay-inset` so nothing is drawn over the titlebar, and
   centred dialogs centre on the region below it. Sheets size to that region
   too. Both that inset and the titlebar's own height come from a single
   `--titlebar-height` token in `app.css`, so the two cannot drift apart. The
   About window redeclares the inset as `0px` since it has no titlebar.

3. **Dismissal exemption** (`src/lib/window-controls.ts`) — the titlebar carries
   `data-window-controls`, and overlay content ignores outside interactions
   originating inside it, so clicking a window control no longer dismisses the
   dialog. Clicking the dimmed page still dismisses, as before.

## Files that carry the changes

| File                                                | Change                                                                                              |
| --------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `ui/dialog/dialog-overlay.svelte`                   | `z-backdrop`, `--overlay-inset` top edge                                                            |
| `ui/sheet/sheet-overlay.svelte`                     | same                                                                                                |
| `ui/alert-dialog/alert-dialog-overlay.svelte`       | same                                                                                                |
| `ui/dialog/dialog-content.svelte`                   | `z-overlay-content`, region-centred `top`, `max-h` + `overflow-y-auto`, `onInteractOutside` wrapper |
| `ui/sheet/sheet-content.svelte`                     | `z-overlay-content`, `--overlay-inset`/`h-[calc(...)]` per side, `onInteractOutside` wrapper        |
| `ui/alert-dialog/alert-dialog-content.svelte`       | same as dialog-content                                                                              |
| `ui/dropdown-menu/dropdown-menu-content.svelte`     | `z-floating`, `max-h-(--bits-floating-available-height)`                                            |
| `ui/dropdown-menu/dropdown-menu-sub-content.svelte` | `z-floating`                                                                                        |
| `ui/popover/popover-content.svelte`                 | `z-floating`                                                                                        |
| `ui/select/select-content.svelte`                   | `z-floating`                                                                                        |
| `ui/tooltip/tooltip-content.svelte`                 | `z-floating`                                                                                        |

Supporting app code (not vendored): `title-bar.svelte`, `window-controls.ts`, `routes/+layout.svelte`.
