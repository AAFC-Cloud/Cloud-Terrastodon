# Interactive Excalidraw files

For object exploration and discovery planning, use
[`ct ratatui --excalidraw …`](object-explorer.md#excalidraw-rendering-and-discovery-planning).
Ctrl-O renders a slot and Ctrl-U renders the current tab; the terminal owns the
inputs. The command below is the separate file-synchronization playground.

```powershell
ct excalidraw interactive .\live.excalidraw
```

This local playground watches a plain `.excalidraw` JSON drawing and publishes
changes through that file. It does not authenticate or call cloud APIs. The
parent directory must exist; a missing drawing is initialized with a demo board.
Existing drawings keep their content and layout.

Open the file in the AAFC-Cloud Excalidraw VS Code fork with desktop file
synchronization support. Save changes, or enable VS Code Auto Save, to make them
visible to CT. Unsaved canvas changes remain local to the editor. The extension
does not launch CT or establish a process/network connection to it.

After installing an updated VSIX, save open drawings and run **Developer: Reload
Window**. Installing the package does not replace an extension host that is
already running. The desktop host supplies local file watching; an older browser
host can continue displaying the drawing without observing CT's file updates
until the window reloads.
The **Excalidraw** Output channel reports the active runtime, watched file path,
external updates, and any merge conflicts. Use it to check that desktop file
synchronization is enabled in the window where the drawing is open.

## Terminal interaction

The terminal stays on its normal screen. A small Ratatui inline viewport shows
the file, recognized question answers, selected canvas log level, and diagnostics.
Tracing records use the same formatter as ordinary terminal output, including
colors, fields, source locations, and spans. The terminal owner makes room above
Ratatui's [inline viewport](https://ratatui.rs/examples/apps/inline/) to print
those formatted records into scrollback, then redraws the panel. Tracing does not
write competing bytes into the panel.

- **Space** changes the visible `#CTLiveReload` status to test external reloading.
- **Q** or **Esc** closes the session.
- **Ctrl-C** cancels it, waiting for an atomic publication already in progress.

Redirected input/output disables the terminal panel and keyboard shortcuts; the
file session continues until cancellation. Files are polled roughly three times
per second, so atomic replacement and delete/recreate are observed without relying
on one filesystem event per edit. Unchanged snapshots do not cause new writes.

## Visible controls

The demo contains a rectangle with two separate text elements:
`#UserQuestion1` and `waffles`. Change the answer and save; the terminal shows
`#UserQuestion1 -> your answer`. Additional unique `#UserQuestion…` labels work
the same way. Each label needs one enclosing rectangle and one answer text.
Duplicate identifiers, overlapping ambiguous containers, deleted controls, and
unsupported rotated geometry are diagnosed rather than guessed.

The native Excalidraw frame named **Logs** contains five labeled target rectangles:
**Debug**, **Info**, **Warn**, **Error**, and **Off**. Move the rectangle with
the bound **Log Level** label onto a target and save. Its center must identify one
target. This changes only which messages appear in the canvas. Normal terminal
and file filters, such as `--log-filter warn`, continue to apply independently.

When CT publishes the filtered logs, the movable selector's border, fill, and
label adopt the selected level's palette as acknowledgement. Its position stays
where you placed it. Until another valid selection is published, it retains the
last acknowledged colour; ambiguous placement does not produce an acknowledgement.

Canvas capture includes Debug through Error, even when the terminal filter is
Info. The session retains the latest 512 records from a bounded tracing capture
buffer. Below `#Logs`, each event has separate native text elements for its
level, message, and fields. Levels and selector targets share a palette: blue
Debug, green Info, amber Warn, red Error, and gray Off. Fields use muted text.
Existing drawings receive the palette and structured rows on their next update.

CT preserves complete message and field text and sets element widths. The fork's
renderer uses Excalidraw's own font measurement and wrapping, then stacks the
newest rows that fit below `#Logs`. A frame's optional `customData.layout` row
hint governs this presentation only; visible labels remain the input contract.
Without that renderer support, another editor still sees ordinary text elements
at their fallback positions. Install the current fork and reload the window to
enable native text measurement and row placement.

Resize the Logs frame and the `#Logs` label's width to change the available area.
CT sends at most 16 candidate rows and reuses their elements, retaining deletion
tombstones without growing the scene indefinitely. **Off** clears the generated
rows. A newest event taller than the whole area is clipped by the native frame;
its full text remains in the drawing. Wrapping and layout alone do not dirty the
VS Code document or discard older hidden candidate rows on save.

Existing drawings can use these same visible conventions. Space appends a reload
status if one does not exist; CT does not insert a whole demo board into an existing
drawing. Ambiguous controls are left untouched. These controls demonstrate file
interaction; they do not approve or execute registry operations.

## Shared-file protocol

The desktop extension and CT cooperate on local files:

1. Resolve the drawing's canonical path so directory-junction aliases share a lock.
2. Exclusively create `<drawing>.sync-lock` with version/owner/process diagnostics.
3. Reread the current drawing while holding the lock and apply pending edits.
4. Flush a complete temporary file beside it, then atomically replace the drawing.
5. Release the lock.

Readers open and read one complete old or new snapshot. Atomic replacement prevents
partial JSON; the shared lock prevents cooperating writers from overwriting one
another's publications. CT retries lock contention while retaining pending edits.
It preserves unknown properties, unfamiliar element types, images, and tombstones.
An invalid or temporarily missing drawing leaves the last valid view intact; CT
waits for the file to be repaired or restored instead of replacing it with a demo.

The extension separately merges the last accepted disk snapshot, unsaved editor
changes, and new disk content. Conflicting edits require resolution. A lock left
by a crashed process is not stolen merely because time has elapsed: establish that
the writer has stopped before removing that specific lock. Other editors must use
the protocol to receive the same simultaneous-write guarantees. Network/virtual
filesystem providers and SVG/PNG are outside this initial shared-file contract.

This command complements [`ct dump plan`](dump-planning.md). Live cloud discovery,
plan selection from the canvas, and durable execution decisions remain separate
work; this playground establishes the file, terminal, and canvas interaction loop.
