# Windows resize investigation — 1.5.2-alpha, revised 2026-11-01

## Evidence and scope

The supplied Windows recording shows trails covering the whole window, including
its GTK-drawn title bar, when dragging the borders of an almost empty document.
The user retested the initial 1.5.2-alpha Cairo workaround and reported no improvement.
Neither observation identifies the precise failing GTK/driver/compositor code.
There is no evidence here that GtkSourceView's text layout is the cause.

The application sets its initial size once. It does not resize the top-level
window in response to size notifications. Its custom editor drawing is confined
to the editor widget, whereas the recording also shows artifacts in decorations.

## Upstream findings

- GTK documents `GTK_CSD=0` as delegating decoration to the window manager for
  windows without a custom titlebar:
  https://docs.gtk.org/gtk4/running.html#gtk_csd
- In GTK 4.24.1, `gtk_window_should_use_csd()` checks that setting on Win32.
  `gtk_window_realize()` otherwise creates the default GTK header bar:
  https://github.com/GNOME/gtk/blob/4.24.1/gtk/gtkwindow.c
- `gdk_win32_surface_do_move_resize_drag()` implements the custom decoration's
  drag geometry. Native frame interactions use Windows sizing messages and the
  modal sizing loop handled by GDK:
  https://github.com/GNOME/gtk/blob/4.24.1/gdk/win32/gdksurface-win32.c
  https://github.com/GNOME/gtk/blob/4.24.1/gdk/win32/gdkevents-win32.c
- Cairo is a renderer, not a guarantee that Windows presentation avoids the GPU.
  GTK 4.24.1's Cairo backend has a DirectComposition/D3D11 path, including swap
  chain presentation. The earlier explanation that Cairo bypassed all GPU
  composition was inaccurate:
  https://github.com/GNOME/gtk/blob/4.24.1/gdk/win32/gdkcairocontext-win32.c
- GTK's release notes also contain Win32/Cairo resize bug #7541. That establishes
  that Cairo is not a universal resize fix, but does **not** establish that this
  Leditor report is the same bug or justify downgrading GTK:
  https://github.com/GNOME/gtk/blob/4.24.1/NEWS

## Implemented correction

Windows launches now default to native title bars/borders before GTK starts.
This avoids GTK's custom decoration drag path and uses the native Windows frame.
The ineffective forced renderer selection is removed. Explicit diagnostic
environment overrides are respected. There are no new dependencies, timers or
Win32 message hooks. The version stays 1.5.2-alpha and the enlarged icon is retained.

The internal editor layout, shortcuts and light/dark palettes remain unchanged;
the outer Windows title bar uses the system's styling. Linux remains unchanged.
This is an evidence-based compatibility correction, not a demonstrated upstream
root-cause fix. Native Windows visual confirmation remains necessary.

## Validation on the affected Windows machine

1. Close every old Leditor window and install with `-Source .` from this revised
   ZIP. The same version number alone cannot distinguish the two builds.
2. Launch from the updated Start menu entry. The main title bar should now be
   Windows-native, not the centered GTK header bar. An explicit `GTK_CSD=1`
   override prevents this change from taking effect.
3. Drag each of the four edges and four corners slowly and quickly, enlarging
   and shrinking. Watch both the content and the exposed desktop for trails.
4. Maximize/restore, snap to the sides, and enter/leave Focus with Shift+F11/Esc.
   Repeat in both editor themes and, if available, at 100% and 150% display scale.
5. Repeat with text, search visible, explorer/minimap enabled, then image/PDF tabs.
   Text, caret, selection, dirty state and zoom must remain intact.

If trails persist even with a visibly native title bar, the decoration hypothesis
is insufficient. Preserve a new recording and the installed GTK version from
`LICENSES/native-packages.txt`, Windows version/build, GPU and driver version,
display scale, and any desktop/window effects. That evidence is needed to
separate an application issue from GTK presentation, driver or desktop effects.

## Follow-up: user report, 2026-11-01

Windows 11 build 26200, GTK 4.24.1, AMD RX 6600, 180 Hz. Windows already
enables showing contents while dragging. All three cases exited successfully
but the user reported blank exposed client areas during resize in all of them.
The recording confirms a native frame growing beyond the old client drawing.

The default renderer already fell back to Cairo: GL and Vulkan reported that
they require DirectComposition. Case 2 rejected GDK_DISABLE=dcomp; it is not
a validated independent disable test. Case 3 accepted GDK_DEBUG=dcomp and
selected Cairo, with no reported initialization failure. These logs do not
independently prove which presentation path every frame used.

Frame-clock logs contain maximum inter-frame intervals of 3068.1, 2411.1 and
1411.1 ms respectively. Idle applications need not paint continuously, so
these maxima alone are NOT resize-stall measurements or a performance benchmark.
CSV traces observed native move/resize in 92/154, 196/289 and 221/278 samples.
They measure HWND client geometry, not GTK widget allocations. Exact root cause
remains unresolved: these records cannot separate frame scheduling, layout and
presentation conclusively. There is no evidence sufficient to blame the GPU
driver, GtkSourceView or missing D-Bus.

User-authorized mitigation: native frame + Cairo + DirectComposition on Windows.
Set GTK_CSD=0, GSK_RENDERER=cairo and GDK_DEBUG=dcomp before GTK initializes,
only when each variable is absent. No frame-debug logging is enabled. Explicit
overrides remain respected. Linux and version 1.5.2-alpha are unchanged.
This supersedes the renderer-default policy described above. No guaranteed
fix or native Windows validation of this source revision is claimed.
