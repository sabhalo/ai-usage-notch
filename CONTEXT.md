# AI Usage Notch

A menu-bar-adjacent Tauri app that shows live usage percentages for several AI provider accounts (Claude, Codex, Copilot) in a small floating pill, with a per-provider detail view on demand.

## Language

**Pill**:
The always-on-top floating window itself, showing one ring per active provider. Has two independent states, Visibility and Panel — never conflate them.
_Avoid_: notch, widget, bar

**Visible** (Pill visibility):
The Pill at full size: every provider's ring and percentage rendered. The resting state in "always visible" mode, and what the Pill returns to when it wakes from Collapsed.
_Avoid_: expanded, extended, open (reserve "open" for the Panel)

**Collapsed** (Pill visibility):
The Pill shrunk to a mini tab: rings only, no percentage text. Reached after a configurable period of inactivity, only when Visibility mode is Auto-collapse. Replaces the retired "compact" density toggle outright — there is no third density, and a Pill cannot be both Collapsed and have an open Panel.
_Avoid_: compact, mini, hidden

**Wake**:
The transition from Collapsed back to Visible, triggered by hovering the Pill. Not called "expand" — that word is retired from this vocabulary because it used to describe the Panel opening, a different axis.

**Visibility mode** (setting):
Governs whether the Pill can ever become Collapsed: **Always** (never collapses) or **Auto-collapse** (collapses after N seconds of inactivity, N configurable).
_Avoid_: display mode, density

**Scale** (setting):
A continuous multiplier (70%–200%, default 100%) applied to every measurement of the Pill and its Panel. Orthogonal to Visibility and Panel — it scales the Visible state, the Collapsed state and the Panel alike. Not a density: it does not reintroduce the retired "compact" axis, and it adds no new state.
_Avoid_: density, compact, zoom, size mode

**Panel**:
The dropdown shown below the Pill with one provider's detail rows (per-window usage bars, reset countdowns). Independent of Pill visibility: opening the Panel always forces the Pill to Visible, and the Pill cannot collapse while the Panel is Open.
_Avoid_: detail view, drawer, dropdown (as a state name — "dropdown" is fine as a passing visual description, not as the state word)

**Open / Closed** (Panel states):
Whether the Panel is currently shown. Distinct from Pill Visible/Collapsed — a state name from one axis should never be used to describe the other.
_Avoid_: expanded/collapsed (retired for this axis to stop the historical overlap with Pill visibility)
