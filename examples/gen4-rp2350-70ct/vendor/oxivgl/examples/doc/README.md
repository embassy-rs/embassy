# oxivgl — Examples

LVGL example screens ported from the [LVGL docs](https://docs.lvgl.io/9.5/examples.html).

Each example is a self-contained file with a `View` impl and a cfg-gated
runner (`example_main!` macro selects host SDL2 or ESP32 fire27 backend).

## Contents

- [Getting Started](#getting-started)
- [Styles](#styles)
- [Animations](#animations)
- [Events](#events)
- [Layouts — Flex](#layouts--flex)
- [Layouts — Grid](#layouts--grid)
- [Scrolling](#scrolling)
- [Widgets — Base Object](#widgets--base-object)
- [Widgets — Animation Image](#widgets--animation-image)
- [Widgets — Arc](#widgets--arc)
- [Widgets — ArcLabel](#widgets--arclabel)
- [Widgets — Image](#widgets--image)
- [Widgets — Bar](#widgets--bar)
- [Widgets — Button](#widgets--button)
- [Widgets — Checkbox](#widgets--checkbox)
- [Widgets — Dropdown](#widgets--dropdown)
- [Widgets — Label](#widgets--label)
- [Widgets — LED](#widgets--led)
- [Widgets — Line](#widgets--line)
- [Widgets — List](#widgets--list)
- [Widgets — Menu](#widgets--menu)
- [Widgets — Msgbox](#widgets--msgbox)
- [Widgets — Chart](#widgets--chart)
- [Widgets — Buttonmatrix](#widgets--buttonmatrix)
- [Widgets — Keyboard](#widgets--keyboard)
- [Widgets — Roller](#widgets--roller)
- [Widgets — Scale](#widgets--scale)
- [Widgets — Slider](#widgets--slider)
- [Widgets — Switch](#widgets--switch)
- [Widgets — Textarea](#widgets--textarea)
- [Widgets — Canvas](#widgets--canvas)
- [Widgets — Calendar](#calendar)
- [Widgets — AnimImg](#widgets--animimg)
- [Widgets — Span](#widgets--span)
- [Widgets — Tileview](#widgets--tileview)
- [Widgets — Imagebutton](#widgets--imagebutton)
- [Widgets — Win](#widgets--win)
- [Widgets — Lottie](#widgets--lottie-abandoned)
- [Widgets — Spinbox](#widgets--spinbox)
- [Widgets — Spinner](#widgets--spinner)
- [Observer](#observer)
- [Gradients](#gradients)
- [Snapshot](#snapshot)
- [Implementation Coverage](#implementation-coverage)
- [Running](#running)

## Getting Started

### Example 1 — Hello World

Dark blue screen, centered white label.

![getting_started1](screenshots/getting_started1.png)

### Example 2 — Button

Default-styled button with a centered label.

![getting_started2](screenshots/getting_started2.png)

### Example 3 — Custom styles

Two buttons with hand-crafted gradient styles and a darken press filter.
Button 2 uses a fully-rounded (pill) radius.

![getting_started3](screenshots/getting_started3.png)

### Example 4 — Slider

Centered slider; label above shows current value, updated live on
`VALUE_CHANGED` events.

![getting_started4](screenshots/getting_started4.png)

### Example 5 — Simple Horizontal Gradient

Container with a horizontal red→green gradient (opacity 100%→0%), stops at 20% and 80%.

![getting_started5](screenshots/getting_started5.png)

### Example 6 — Linear (Skew) Gradient

Container with a skewed linear gradient from (100,100) to (200,150).

![getting_started6](screenshots/getting_started6.png)

### Example 7 — Radial Gradient

Container with a radial gradient centered at (100,100), focal point at (50,50).

![getting_started7](screenshots/getting_started7.png)

### Example 8 — Conical Gradient

Container with a conical gradient sweeping 0°–180° from center.

![getting_started8](screenshots/getting_started8.png)

## Styles

### Style 1 — Size, Position and Padding

Object with explicit width, content-based height, padding, and percentage position.

![style1](screenshots/style1.png)

### Style 2 — Background Gradient

Centered object with a two-stop vertical gradient (shifted towards bottom).

![style2](screenshots/style2.png)

### Style 3 — Border

Centered object with bottom+right blue border on light grey background.

![style3](screenshots/style3.png)

### Style 4 — Outline

Centered object with blue outline and 8px padding around it.

![style4](screenshots/style4.png)

### Style 5 — Shadow

Centered object with a large blue drop shadow.

![style5](screenshots/style5.png)

### Style 6 — Image Style Properties

Cogwheel image rotated 30°, blue recolor tint on grey background with blue border.

![style6](screenshots/style6.png)

### Style 7 — Arc

Arc widget with red color and 4px width.

![style7](screenshots/style7.png)

### Style 8 — Text Styles

Text with blue color, letter spacing, line spacing, and underline decoration.

![style8](screenshots/style8.png)

### Style 9 — Line Styles

Grey rounded polyline with 6px width.

![style9](screenshots/style9.png)

### Style 10 — Transition

Object with animated transitions on press (bg color, border color/width).

![style10](screenshots/style10.png)

### Style 11 — Multiple Styles

Base (light blue) and Warning (yellow) objects sharing a common style with overrides.

![style11](screenshots/style11.png)

### Style 12 — Local Styles

Green-bordered object with local orange background override.

![style12](screenshots/style12.png)

### Style 13 — Parts and States

Slider with gradient indicator and red shadow on pressed state.

![style13](screenshots/style13.png)

### Style 14 — Extending the Current Theme

Two buttons: the first uses the default theme; after installing a theme extension
the second button is styled green with a dark border automatically by the theme machinery.

![style14](screenshots/style14.png)

### Style 15 — Opacity and Transformations

Three buttons: normal (100%), 50% opacity, and 50% opacity with 15° rotation and 1.25× scale.
Host screenshot shows opacity only; transforms render correctly on hardware.

![style15](screenshots/style15.png)

### Style 16 — Conical Gradient

Metallic knob using 8-stop conical gradient with reflected extend and drop shadow.

![style16](screenshots/style16.png)

### Style 17 — Radial Gradient

Full-screen radial gradient from purple to black.

![style17](screenshots/style17.png)

### Style 18 — Gradient Buttons

Four buttons: simple horizontal, simple vertical, complex linear, complex radial gradients.

![style18](screenshots/style18.png)

### Style 20 — Modal Overlay Dimming

A screen with two buttons ("BG Dim" and "OPA Dim"). A full-screen dark overlay
is shown on top with a dismiss label.

![style20](screenshots/style20.png)

### Style 21 — Material-design cards with shadow, rotation & scale transforms

Two card objects with rounded corners and drop shadow. An Arc controls
rotation transform; a Slider controls scale transform on both cards.

![style21](screenshots/style21.png)

### Shared Styles 1 — One style guide applied to many widgets

The memory-efficient styling pattern: each visual treatment is built **once** as
a shared `Style` (via `Style::new`) and applied to every widget with `add_style`
— zero inline style setters. See [memory-tuning.md](../../docs/memory-tuning.md).

![shared_styles1](screenshots/shared_styles1.png)

### Skipped

- **Style 19** — Modal overlay (meta-example, benchmarking)

## Animations

### Anim 1 — Start Animation on Event

Switch toggles label X-position animation (overshoot/ease-in paths).

![anim1](screenshots/anim1.png)

### Anim 2 — Playback Animation

Red circle with repeat/reverse size + X animations (ease-in-out).

![anim2](screenshots/anim2.png)

### Anim 3 — Cubic Bezier with Chart

Two sliders (P1, P2) adjust bezier control points. A scatter chart shows the
curve in real-time. Click the play button to animate a red square along the
current bezier curve.

![anim3](screenshots/anim3.png)

### Anim 4 — Animation with Timed Pause

Switch toggles label X animation (overshoot / ease-in). A one-shot 200 ms timer
pauses the running animation for 1 s.

![anim4](screenshots/anim4.png)

### Anim Timeline 1 — Animation Timeline

Three objects animated via timeline, controlled by start/pause buttons and a progress slider.

![anim_timeline1](screenshots/anim_timeline1.png)

## Events

All event examples use the safe `View::on_event` dispatch — no `unsafe` or raw callbacks in user code.

> **Note:** Hardware target (fire27) has no touch screen yet — input events require a physical input device. The GUI is fully wired; only the physical input is missing.

### Event Click — Button Click Counter

Button increments a counter label on each click.

![event_click](screenshots/event_click.png)

### Event Button — Multiple Event Types

Button reports pressed, clicked, long-pressed, and long-pressed-repeat events to an info label.

![event_button](screenshots/event_button.png)

### Event Bubble — Event Bubbling

30-button grid in a flex container; clicking any button turns it red via bubbled events.

![event_bubble](screenshots/event_bubble.png)

### Event Trickle — Event Trickle-Down

9-cell grid with trickle-down: pressing the container applies a black style to all children.

![event_trickle](screenshots/event_trickle.png)

### Event Streak — Short-Click Streak Counting

Button reports short-clicked (with streak count), single-clicked, double-clicked, and triple-clicked events to labels.

![event_streak](screenshots/event_streak.png)

### Event Draw — Custom Draw Task

Container with `DRAW_TASK_ADDED` events. `update()` cycles a size value 0→50→0;
the draw handler draws a growing/shrinking circle centered in the container.

![event_draw](screenshots/event_draw.png)

## Layouts — Flex

### Flex 1 — Row and Column

Row container (scrollable) and column container with 10 buttons each.

![flex1](screenshots/flex1.png)

### Flex 2 — Row Wrap with Even Spacing

Style-based flex config: row-wrap flow with `SPACE_EVENLY` alignment. Items are checkable.

![flex2](screenshots/flex2.png)

### Flex 3 — Flex Grow

Fixed-size items alongside items with `flex_grow` (1 and 2 portions of free space).

![flex3](screenshots/flex3.png)

### Flex 4 — Column Reverse

Items added 0–5 but displayed bottom-to-top via `ColumnReverse` flow.

![flex4](screenshots/flex4.png)

### Flex 5 — Row and Column Gap

Row-wrap layout with animated `pad_row` (500 ms) and `pad_column` (3000 ms) gap changes.

![flex5](screenshots/flex5.png)

### Flex 6 — RTL Direction

Right-to-left base direction reverses item order in a row-wrap container.

![flex6](screenshots/flex6.png)

## Layouts — Grid

### Grid 1 — Simple Grid

3×3 grid with fixed 70 px columns and 50 px rows, stretched button cells.

![grid1](screenshots/grid1.png)

### Grid 2 — Cell Placement and Span

Different cell alignments (START, CENTER, END) and multi-column/row spanning.

![grid2](screenshots/grid2.png)

### Grid 3 — Free Units (FR)

Column 1 fixed 60 px, column 2 gets 1 FR, column 3 gets 2 FR of remaining space.

![grid3](screenshots/grid3.png)

### Grid 4 — Track Placement

`SPACE_BETWEEN` columns, rows aligned to `END` (bottom).

![grid4](screenshots/grid4.png)

### Grid 5 — Column and Row Gap

3×3 grid with animated row gap (500 ms) and column gap (3000 ms).

![grid5](screenshots/grid5.png)

### Grid 6 — RTL Direction

Same 3×3 grid but with `RTL` base direction — cells fill right-to-left.

![grid6](screenshots/grid6.png)

## Scrolling

### Scroll 1 — Basic Scrolling with Save/Restore

Panel with children placed outside its bounds triggers automatic scrolling.
Two buttons save and restore the scroll position.

![scroll1](screenshots/scroll1.png)

### Scroll 2 — Scroll Snap

Horizontal row of buttons with center snap alignment. Panel 3 is non-snappable.
A switch toggles "scroll one" mode.

![scroll2](screenshots/scroll2.png)

### Scroll 3 — Floating Button in List

A list with initial tracks and a floating "+" button. Clicking the button
adds a new track and scrolls it into view.

![scroll3](screenshots/scroll3.png)

### Scroll 4 — Scrollbar Styling

Custom blue rounded scrollbar that widens and becomes fully opaque when
actively scrolling, with animated transitions.

![scroll4](screenshots/scroll4.png)

### Scroll 5 — Right-to-Left Scrolling

A container with RTL base direction containing a wide label with Persian text.
The text scrolls from right to left using the DejaVu 16 Persian/Hebrew font.

![scroll5](screenshots/scroll5.png)

### Scroll 6 — Curved Scroll

A circular clipped flex column where items are displaced horizontally based on
their distance from the container centre (circle arc formula). Items far from
centre are also made more transparent.

![scroll6](screenshots/scroll6.png)

### Scroll 7 — Dynamic Widget Loading

A scrollable column that dynamically loads and unloads items as the user scrolls.
Labels show the current range of loaded item numbers. A checkbox toggles scrollbar
visibility.

![scroll7](screenshots/scroll7.png)

### Scroll 8 — Circular Scroll

Two scroll containers (horizontal row, vertical column) that loop infinitely:
when the list reaches either end, the last/first item is moved to the opposite
end and the scroll position is adjusted.

![scroll8](screenshots/scroll8.png)

### Scroll 9 — Scroll Property Toggles

A scrollable panel with colored child objects and 4 switches that toggle
scroll flags (SCROLLABLE, CHAIN, ELASTIC, MOMENTUM).

![scroll9](screenshots/scroll9.png)

### Skipped

- **Scroll 5** — RTL scrolling (needs `LV_FONT_DEJAVU_16_PERSIAN_HEBREW`)

## Widgets — Base Object

### Widget Obj 1 — Base Objects with Custom Styles

Two base objects: a plain one and one with a blue shadow style.

![widget_obj1](screenshots/widget_obj1.png)

### Widget Obj 3 — Matrix Transform Animation

Centered object with animated scale + rotation via 3×3 matrix transform.

![widget_obj3](screenshots/widget_obj3.png)

### Widget Obj 2 — Draggable Object

A base object that follows the pointer when pressed, using indev movement vector.

![widget_obj2](screenshots/widget_obj2.png)

## Widgets — Animation Image

### animimg_1 — Animated image cycling through frames

`AnimImg` widget with a two-frame animation using the cogwheel image asset.
Both frames use the same image; in a real application each frame would be
distinct. The animation loops infinitely with a 1-second cycle.

![animimg_1](screenshots/animimg_1.png)

## Widgets — Arc

### Widget Arc 1 — Arc with Value Label

Arc with VALUE_CHANGED event; a label follows the arc's knob angle via
`rotate_obj_to_angle` (positions and rotates the label along the arc edge).

![widget_arc1](screenshots/widget_arc1.png)

### Widget Arc 2 — Animated Arc Loader

Full-circle arc animating 0→100 in 1 s (infinite repeat, 500 ms delay). Knob hidden, not clickable.

![widget_arc2](screenshots/widget_arc2.png)

### Widget Arc 3 — Interactive Pie Chart

Interactive pie chart with click-to-pop-out animation.

![widget_arc3](screenshots/widget_arc3.png)

## Widgets — ArcLabel

### Widget ArcLabel 1 — Text curved along circular arcs

Three ArcLabel widgets with different radius, angle, and direction settings
(clockwise and counter-clockwise).

![widget_arclabel1](screenshots/widget_arclabel1.png)

## Widgets — Image

### Widget Image 1 — Basic Image Display

Centered cogwheel image from compiled PNG asset.

![image1](screenshots/image1.png)

### Widget Image 3 — Rotating Image

Cogwheel image rotating continuously via `update()`, using `set_rotation` and `set_pivot`.

![widget_image3](screenshots/widget_image3.png)

### Widget Image 2 — Runtime Image Recoloring

Cogwheel image with RGB + intensity sliders controlling recolor tint.

![widget_image2](screenshots/widget_image2.png)

### Widget Image 4 — Image Offset Animation

Stripe image with yellow background, black recolor, and animated vertical offset.

![widget_image4](screenshots/widget_image4.png)

### Widget Image 5 — Image Inner Alignment

Three images showing different inner alignment modes: default, stretch, tile.

![widget_image5](screenshots/widget_image5.png)

## Widgets — Bar

### Widget Bar 1 — Simple Bar

Simple 200×20 bar at 70%.

![widget_bar1](screenshots/widget_bar1.png)

### Widget Bar 2 — Styled Progress Bar

Blue-themed bar with custom bg/indicator styles, rounded corners, padding, and animated fill.

![widget_bar2](screenshots/widget_bar2.png)

### Widget Bar 3 — Temperature Meter

Vertical bar with red-to-blue gradient indicator, animated between -20 and 40 (3 s each direction).

![widget_bar3](screenshots/widget_bar3.png)

### Widget Bar 4 — Stripe Pattern

Range-mode bar with tiled stripe background image on the indicator at 30% opacity.

![widget_bar4](screenshots/widget_bar4.png)

### Widget Bar 5 — LTR vs RTL Bars

Two bars: one left-to-right (default), one right-to-left, with labels.

![widget_bar5](screenshots/widget_bar5.png)

### Widget Bar 6 — Bar with Custom Draw Value Label

A bar animating 0→100→0 (4 s each). A `DRAW_MAIN_END` handler draws the current
value as text: white inside the indicator when wide enough, black outside on the
right when the indicator is short.

![widget_bar6](screenshots/widget_bar6.png)

### Widget Bar 7 — Reversed Vertical Bar

Vertical bar filling top-to-bottom via reversed range (100→0), at 70%.

![widget_bar7](screenshots/widget_bar7.png)

### Widget Bar 8 — Recolored Stripe Pattern

Range-mode bar with the tiled stripe background image tinted green via `bg_image_recolor` + `bg_image_recolor_opa`.

![widget_bar8](screenshots/widget_bar8.png)

## Widgets — Button

### Widget Button 1 — Click and Toggle

Standard button logging clicks and a checkable toggle button logging state changes.

![widget_button1](screenshots/widget_button1.png)

### Widget Button 2 — Styled Button from Scratch

Button with gradient, shadow, outline, and a transition that expands outline on press.

![widget_button2](screenshots/widget_button2.png)

### Widget Button 3 — Gum Squeeze Animation

Button with transform width/height transitions on press — overshoot easing on release.

![widget_button3](screenshots/widget_button3.png)

## Widgets — Checkbox

### Widget Checkbox 1 — Simple Checkboxes

Four checkboxes: unchecked, checked, disabled, and checked+disabled.

![widget_checkbox1](screenshots/widget_checkbox1.png)

### Widget Checkbox 2 — Radio Button Groups

Two independent groups of checkboxes acting as radio buttons via event bubbling.
Clicking one unchecks the rest in its group.

![widget_checkbox2](screenshots/widget_checkbox2.png)

## Widgets — Dropdown

### Widget Dropdown 1 — Simple Drop-Down

Dropdown with ten fruit options; a VALUE_CHANGED event handler updates a selection label below.

![widget_dropdown1](screenshots/widget_dropdown1.png)

### Widget Dropdown 2 — Four Directions

Four dropdowns opening in each cardinal direction (down, up, right, left).

![widget_dropdown2](screenshots/widget_dropdown2.png)

### Widget Dropdown 3 — Menu-Style Dropdown

Dropdown with fixed "Menu" button text and no selected-item highlight.

![widget_dropdown3](screenshots/widget_dropdown3.png)

## Widgets — Label

### Widget Label 1 — Wrap and Scroll

Wrapped centered text and a circularly scrolling label.

![widget_label1](screenshots/widget_label1.png)

### Widget Label 2 — Text Shadow

Fake shadow via duplicate label offset by 2 px with reduced opacity.

![widget_label2](screenshots/widget_label2.png)

### Widget Label 3 — Mixed LTR, RTL and CJK

Three labels: English (LTR), Hebrew (RTL with DejaVu font), Chinese (Source Han Sans).
Demonstrates bidirectional text support.

![widget_label3](screenshots/widget_label3.png)

### Widget Label 4 — Gradient Text via Canvas Mask

Text rendered through an L8 bitmap mask on a gradient background.

![widget_label4](screenshots/widget_label4.png)

### Widget Label 5 — Circular Scroll

Label with scroll-circular long mode — text scrolls in a continuous loop.

![widget_label5](screenshots/widget_label5.png)

### Widget Label 6 — Fixed-Width Font Override

Two labels: proportional Montserrat 20 and monospaced override via
`FixedWidthFont` glyph callback.

![widget_label6](screenshots/widget_label6.png)

### Widget Label 7 — Manual Language Switching

Dropdown selects English/Deutsch/Espanol; four labels update text from
static translation tables. Simplified from LVGL's `lv_translation_*` API
(not enabled in current bindings).

![widget_label7](screenshots/widget_label7.png)

## Widgets — LED

### Widget LED 1 — Brightness and Color

Three LEDs: off (dark), dim red (brightness 150), and full on (blue).

![widget_led1](screenshots/widget_led1.png)

## Widgets — Line

### Widget Line 1 — Styled Line

Blue line through 5 points with 8px width and rounded ends.

![widget_line1](screenshots/widget_line1.png)

## Widgets — List

### Widget List 1 — File/Connectivity/Exit Sections

A list with text section headers and icon+text buttons. Clicking a button
identifies it by text via the View event handler.

![widget_list1](screenshots/widget_list1.png)

### Widget List 2 — Reorderable List

Left panel with 15 selectable items. Right panel with Top/Up/Center/Down/Bottom/Shuffle
control buttons to reorder the selected item.

![widget_list2](screenshots/widget_list2.png)

## Widgets — Menu

### Widget Menu 1 — Simple Menu with Sub-Page

Full-screen menu with a main page containing three items. The third item
navigates to a sub-page.

![widget_menu1](screenshots/widget_menu1.png)

### Widget Menu 2 — Root Back Button with Msgbox

Like menu1 but with the root back button enabled. Clicking the back button
at root level shows a message box.

![widget_menu2](screenshots/widget_menu2.png)

### Widget Menu 3 — Custom Back Button Text and Titled Pages

Full-screen menu with a "Back" label on the header back button.
Three sub-pages with titles, each reachable from the main page.

![widget_menu3](screenshots/widget_menu3.png)

### Widget Menu 4 — Dynamic Menu with Floating Add Button

A menu with one initial item and a floating "+" button. Each click
adds a new item with a sub-page and scrolls it into view.

![widget_menu4](screenshots/widget_menu4.png)

### Widget Menu 5 — Settings Menu with Sidebar

A full settings UI with sidebar navigation, sections, separators,
sliders, switches, and a sidebar-toggle switch. Root back button
shows a message box.

![widget_menu5](screenshots/widget_menu5.png)

## Widgets — Msgbox

### Widget Msgbox 1 — Standalone Message Box

A modal message box with a title, body text, and a close button.
Msgbox is also used as a supporting widget in menu2 and menu5.

![widget_msgbox1](screenshots/widget_msgbox1.png)

### Widget Msgbox 2 — Settings Dialog

Non-modal message box styled as a settings dialog with minimize and close
header buttons, brightness and speed sliders in the content area, and
Apply/Cancel footer buttons with indigo background.

![widget_msgbox2](screenshots/widget_msgbox2.png)

### Widget Msgbox 3 — Message Box with Backdrop Blur

Modal message box with blur + dimming on layer_top. Background labels show
through the blurred backdrop. Blur requires GPU-accelerated backend; on SDL
host the dimming is visible but blur is a no-op. (Screenshot captures active
screen only — modal layer_top content not visible in PNG.)

![widget_msgbox3](screenshots/widget_msgbox3.png)

## Widgets — Chart

### Widget Chart 1 — Basic Line Chart

Two data series (green on primary Y, red on secondary Y), 10 points each.
Shadow style on data-point items.

![widget_chart1](screenshots/widget_chart1.png)

### Widget Chart 2 — Bar Chart with Two Series

A 200x150 bar chart with 12 monthly data points, division lines, and two colored series.

![widget_chart2](screenshots/widget_chart2.png)

### Widget Chart 3 — Stacked Bar Chart

A 280x180 chart with three stacked series (red, green, blue), 10 points each.

![widget_chart3](screenshots/widget_chart3.png)

### Widget Chart 4 — Bar Chart with Value-Based Coloring

A bar chart where each bar's color is interpolated between red (low) and
green (high) based on its Y value. Uses `DRAW_TASK_ADDED` events with
`with_fill_dsc` closure API.

![widget_chart4](screenshots/widget_chart4.png)

### Widget Chart 6 — Cursor on Clicked Point

A line chart with a cursor crosshair. Clicking a data point moves the cursor
to that location.

![widget_chart6](screenshots/widget_chart6.png)

### Widget Chart 7 — Scatter Chart with Live Data

A scatter plot with 50 data points. A timer adds new random points every 100 ms.

![widget_chart7](screenshots/widget_chart7.png)

### Widget Chart 8 — Circular Line Chart with Gap

A line chart in circular update mode with 80 data points updated every 300 ms.
Three points ahead of the write cursor are blanked to create a visible gap.

![widget_chart8](screenshots/widget_chart8.png)

## Widgets — Buttonmatrix

### Widget Buttonmatrix 1 — Basic Numpad

A 3-row button matrix: digits 1-5, 6-0, and two action buttons.
Action1 is 2x wide and checkable; Action2 starts checked.

![widget_buttonmatrix1](screenshots/widget_buttonmatrix1.png)

### Widget Buttonmatrix 2 — Custom Draw per Button

Default button matrix where three buttons get custom styling via
`DRAW_TASK_ADDED`. Button 2 is blue with a box shadow and square corners,
button 3 is red with circular radius and matching shadow, button 4 hides
its label and shows a star image drawn during the fill pass.

![widget_buttonmatrix2](screenshots/widget_buttonmatrix2.png)

### Widget Buttonmatrix 3 — Pagination

A pill-shaped button group with left/right arrows and numbered pages 1-5.
Only one page number can be checked at a time; arrows navigate between pages.

![widget_buttonmatrix3](screenshots/widget_buttonmatrix3.png)

## Widgets — Keyboard

### Widget Keyboard 1 — Keyboard with Two Textareas

Two textareas at top with placeholder text. A keyboard at the bottom switches
between them on focus events.

![widget_keyboard1](screenshots/widget_keyboard1.png)

### Widget Keyboard 2 — Custom AZERTY Layout

A keyboard using a custom AZERTY key map assigned to User1 mode.

![widget_keyboard2](screenshots/widget_keyboard2.png)

### Widget Keyboard 3 — Per-Key Coloring with Star Icon

Each key gets a unique palette color via `DRAW_TASK_ADDED` draw hooks. The
OK key hides its label and draws a star image in its place.

![widget_keyboard3](screenshots/widget_keyboard3.png)

## Widgets — Roller

### Widget Roller 1 — Month Roller

Infinite roller with month names, 4 visible rows.

![widget_roller1](screenshots/widget_roller1.png)

### Widget Roller 2 — Styled Rollers with Alignments

Three rollers: left-aligned on green gradient, center-aligned, right-aligned. Shared
selected-row style with Montserrat 20pt font and pink/red highlight.

![widget_roller2](screenshots/widget_roller2.png)

### Widget Roller 3 — Roller with Fade Mask

Month roller with vertical gradient mask fading top and bottom rows.

![widget_roller3](screenshots/widget_roller3.png)

## Widgets — Scale

### Widget Scale 1 — Round Gauge

270° round scale with labeled major ticks (0–100), built via `Scale::tick_ring`.

![widget_scale1](screenshots/widget_scale1.png)

### Widget Scale 2 — Horizontal Scale

Horizontal bottom-aligned scale with labeled major ticks from 10 to 40.

![widget_scale2](screenshots/widget_scale2.png)

### Widget Scale 3 — Round Scale with Needle

Round gauge with animated line needle sweeping 0–100.

![widget_scale3](screenshots/widget_scale3.png)

### Widget Scale 4 — Round Scale with Sections

Round outer scale with custom labels 1–10, red section (8–10), green section (1–3).

![widget_scale4](screenshots/widget_scale4.png)

### Widget Scale 5 — Horizontal Scale with Sections

Horizontal scale 0–100 with colored sections: blue (0–25), red (75–100).

![widget_scale5](screenshots/widget_scale5.png)

### Widget Scale 6 — Clock with Timer-Driven Needles

Round clock face with minute and hour hands updated by a 250 ms Timer.

![widget_scale6](screenshots/widget_scale6.png)

### Widget Scale 8 — Round Scale with Rotated Labels

Round inner scale with labels rotated to match tick angles, pink background, and needle.

![widget_scale8](screenshots/widget_scale8.png)

### Widget Scale 9 — Horizontal Scale with Rotated Labels

Horizontal bottom scale with 45° rotated major tick labels.

![widget_scale9](screenshots/widget_scale9.png)

### Widget Scale 10 — Heart Rate Gauge

Round gauge with timer-driven needle oscillating between 80–180 BPM.

![widget_scale10](screenshots/widget_scale10.png)

### Widget Scale 7 — Custom Major Tick Label Color and Text

Horizontal scale with a `DRAW_TASK_ADDED` handler that recolors major tick labels
with a rainbow palette and reformats numeric text as one-decimal floats.

![widget_scale7](screenshots/widget_scale7.png)

### Widget Scale 11 — 24-Hour Clock Face

Round scale with custom hour labels, day/night colored arc sections, and
highlighted cardinal hour labels (06/12/18/24 white, rest grey).

![widget_scale11](screenshots/widget_scale11.png)

### Widget Scale 12 — Compass Gauge with Rotation

Round scale with 8 compass direction labels (N/NE/E/SE/S/SW/W/NW), a red
needle, and continuous rotation animation.

![widget_scale12](screenshots/widget_scale12.png)

## Widgets — Slider

### Widget Slider 1 — Slider with Value Label

Centered slider with a label below showing the current value (updated in `update()`).

![widget_slider1](screenshots/widget_slider1.png)

### Widget Slider 2 — Styled Slider

Cyan slider with pill-shaped track, padded knob with border, bg-color transition on press.

![widget_slider2](screenshots/widget_slider2.png)

### Widget Slider 3 — Range Slider

Range-mode slider with two handles and a label showing min–max values.

![widget_slider3](screenshots/widget_slider3.png)

### Widget Slider 4 — Reversed Slider

Slider with opposite direction (100→0) and percentage label below.

![widget_slider4](screenshots/widget_slider4.png)

## Widgets — Switch

### Widget Switch 1 — Toggle Switches

Four switches in a column: default, checked, disabled, and checked+disabled.

![widget_switch1](screenshots/widget_switch1.png)

### Widget Switch 2 — Horizontal and Vertical

Horizontal switch (default) and vertical switch (pre-checked), using `set_orientation`.

![widget_switch2](screenshots/widget_switch2.png)

## Widgets — Textarea

### Widget Textarea 1 — Numeric Keypad

One-line textarea with a custom numeric button matrix keypad (Buttonmatrix widget).
Pressing digits appends; backspace deletes; enter sends READY.

![widget_textarea1](screenshots/widget_textarea1.png)

### Widget Textarea 2 — Password and Text with Keyboard

Password textarea (left) and plain text textarea (right) with an on-screen Keyboard widget.
Clicking either textarea switches keyboard focus.

![widget_textarea2](screenshots/widget_textarea2.png)

### Widget Textarea 3 — Clock Format Auto-Insert

Textarea restricted to digits and ':', max 5 characters. After two digits, ':' is
auto-inserted via VALUE_CHANGED event. Numeric keyboard below.

![widget_textarea3](screenshots/widget_textarea3.png)

### Widget Textarea 4 — Cursor Styles

Three one-line textareas with unique cursor styles applied via Part::Cursor + ObjState::FOCUSED:
simple red bar, underline blue, and block orange/yellow gradient.

![widget_textarea4](screenshots/widget_textarea4.png)

## Widgets — Canvas

### canvas_1 — Dual canvas + image rotation

Two 100×70 RGB565 canvases. Canvas 1: red filled rect with black border + orange
"Canvas 1" label. Canvas 2: rotated (12°) snapshot of canvas 1 drawn via
`DrawImageDsc`.

![canvas_1](screenshots/canvas_1.png)

### canvas_2 — Transparent pixels

80×60 RGB565 canvas filled blue. Three horizontal bands of decreasing opacity
(50 %, 20 %, 0 %) drawn via `set_px`.

![canvas_2](screenshots/canvas_2.png)

### canvas_3 — Rectangle with border and outline

70×70 RGB565 canvas. Red rectangle with blue border (4 px), green outline (2 px),
and 5 px corner radius, drawn via `CanvasLayer::draw_rect`.

![canvas_3](screenshots/canvas_3.png)

### canvas_4 — Text label

80×30 RGB565 canvas. "Hello" in red via `CanvasLayer::draw_label`.

![canvas_4](screenshots/canvas_4.png)

### canvas_5 — Arc

50×50 RGB565 canvas. Red arc (center 25,25; radius 15; width 10; 0°–220°).

![canvas_5](screenshots/canvas_5.png)

### canvas_6 — Image drawn on canvas

Cogwheel image asset drawn onto a 100×100 ARGB8888 canvas via `DrawImageDsc`.

![canvas_6](screenshots/canvas_6.png)

### canvas_7 — Line

50×50 RGB565 canvas. Red line (15,15)→(35,10), width 4, rounded caps.

![canvas_7](screenshots/canvas_7.png)

### canvas_9 — Gradient triangle

80×80 RGB565 canvas. Semi-transparent triangle (3 vertices) with a vertical
red→blue gradient via `DrawTriangleDsc`.

![canvas_9](screenshots/canvas_9.png)

### canvas_10 — Wavy text animation

160×100 RGB565 canvas. "Hello wavy world!" rendered letter-by-letter with HSV
rainbow colors and sine-wave y-offsets via `DrawLetterDsc`. Animated each frame.

![canvas_10](screenshots/canvas_10.png)

### canvas_11 — Windstorm text animation

160×100 RGB565 canvas (black). "windstorm" repeated with sinusoidal y-positions,
HSV-cycled colors shifting each frame via `DrawLetterDsc`. Animated each frame.

![canvas_11](screenshots/canvas_11.png)

### canvas_12 — Curved text along circular path

240×240 ARGB8888 canvas with "HELLO LVGL 9.5" rendered character-by-character
along a circular arc, each letter rotated tangent to the circle with rainbow
colors. The text orbits continuously.

![canvas_12](screenshots/canvas_12.png)

## Table

### table_1 — Scrollable fruit/price table

2-column, 8-row table (Name / Price) with height=200 (scrollable). A
`DRAW_TASK_ADDED` handler applies: blue-tinted header row with centered text,
right-aligned first column, grey-tinted even non-header rows.

![table_1](screenshots/table_1.png)

### table_2 — Scrollable list with toggle state

Single-column 200-row table used as a lightweight scrollable list. Each row
stores a boolean checked state via `CUSTOM_1` cell-ctrl. A `DRAW_TASK_ADDED`
handler highlights checked rows in blue; `VALUE_CHANGED` toggles the state on
click.

![table_2](screenshots/table_2.png)

## Calendar

### calendar_1 — Month view with highlighted dates and arrow header

February 2021 with three highlighted days (6, 11, and 22 Feb 2022). Arrow
buttons navigate between months. Clicking a day fires `VALUE_CHANGED`; the
label above the calendar updates to show the selected date.

![calendar_1](screenshots/calendar_1.png)

## Widgets — AnimImg

### animimg_1 — Animated image cycling

Cogwheel image in a 2-frame infinite animation.

![animimg_1](screenshots/animimg_1.png)

## Widgets — Span

### span_1 — Rich text with multiple styles

Spangroup with colored, decorated, and differently-sized text spans.

![span_1](screenshots/span_1.png)

## Widgets — Tileview

### tileview_1 — Swipeable tile grid

Three tiles in an L-shaped layout with directional scroll constraints.

![tileview_1](screenshots/tileview_1.png)

## Widgets — Imagebutton

### imagebutton_1 — Stateful image button

Imagebutton with state switching (no visible images without assets).

![imagebutton_1](screenshots/imagebutton_1.png)

## Widgets — Win

### win_1 — Window with header and content

Window with title, close/settings buttons, and scrollable content area.

![win_1](screenshots/win_1.png)

## Widgets — Lottie (abandoned)

Lottie support was investigated and abandoned due to ThorVG binary size impact.

**Problem:** LVGL's Lottie widget depends on ThorVG (C++ vector graphics engine).
Even when no example uses Lottie, enabling `LV_USE_THORVG_INTERNAL` in `lv_conf.h`
links ~240 KB of ThorVG code plus libstdc++, libc, and libgcc into every binary.
The C++ archives bring hundreds of `.gcc_except_table` sections; espflash pads
each to 64 KB alignment, inflating a ~1 MB firmware to >5 MB — exceeding the
default 4 MB app partition. `-fno-exceptions`, `--gc-sections`, and LTO cannot
strip this dead code because `--start-group` (required for cross-archive symbol
resolution) keeps it alive.

## Widgets — Spinbox

### spinbox_1 — Numeric input with +/− buttons

Spinbox with range −1000..25000, 5 digits and 2 decimal places. Plus and minus
buttons increment/decrement the value.

![spinbox_1](screenshots/spinbox_1.png)

## Widgets — Spinner

### spinner_1 — Centered loading spinner

100×100 spinner with a 10 s animation cycle and 200° arc.

![spinner_1](screenshots/spinner_1.png)

### calendar_2 — Chinese calendar with dropdown header (host only)

March 2024 with Chinese lunar day names. Dropdown header for month/year
selection. Requires `LV_USE_CALENDAR_CHINESE` and CJK font.

Note: Chinese day names render on host SDL2 only. On ESP32, LVGL's btnmatrix
widget ignores Part::Items font styles, falling back to the default font
which lacks CJK glyphs. This is an upstream LVGL limitation.

![calendar_2](screenshots/calendar_2.png)

## Tabview

### tabview_1 — Simple 3-tab view with default top bar

Three tabs; Tab 1 has long content that becomes scrollable. Tab 3 is scrolled
into view on creation via `scroll_to_view_recursive`.

![tabview_1](screenshots/tabview_1.png)

### tabview_2 — Left-side bar with 4 tabs

Left-positioned tab bar (width 80 px). Four content tabs with plain text.
Active tab set programmatically to the second tab on startup.

![tabview_2](screenshots/tabview_2.png)

## Observer

### observer1 — Slider bound to a temperature label

Integer subject initialised to 28. Slider centered on screen; label 30 px
below center shows the current value formatted as `"%d °C"`. Moving the
slider updates both the subject and the label automatically.

![observer1](screenshots/observer1.png)

### observer2 — PIN login screen with state bindings

Two subjects: `auth_state_subject` (LOGGED_OUT/LOGGED_IN/AUTH_FAILED) and
`engine_subject` (0/1). A textarea in password mode accepts a PIN; pressing
Enter checks whether it equals `"hello"`. An info label is updated by
polling the auth subject in `update()`. A "LOG OUT" button is disabled when
not logged in via `bind_state_if_not_eq`. A "START ENGINE" checkable button
is two-way bound to `engine_subject` via `bind_checked`.

![observer2](screenshots/observer2.png)

### observer3 — Time display with subject groups (simplified)

Four integer subjects (hour, minute, format, AM/PM) grouped into a single
group subject. Hour/minute rollers and 12/24 format and AM/PM dropdowns are
all bound to the subjects via safe `bind_value()`. The AM/PM dropdown is
disabled in 24-hour mode via `bind_state_if_eq`. The time label is formatted
in `update()` by polling the subject values.

**Simplification**: The C original dynamically creates/deletes a settings
panel on button click. This port shows all controls statically because
dynamic widget creation in event callbacks requires raw LVGL APIs not yet
wrapped.

![observer3](screenshots/observer3.png)

### observer4, observer5, observer6 — Blocked

These examples require infrastructure not yet wrapped:

| Example | Blocker |
|---------|---------|
| observer4 — Tab navigation with animated transitions | Animation callbacks, dynamic widget creation in observer context |
| observer5 — Firmware update state machine | Raw timer callbacks, `lv_obj_clean` in observer, Win content rebuilds |
| observer6 — Light/dark theme switching | Raw style mutation, `lv_obj_report_style_change`, leaked style structs |

## Gradients

### grad_1 — Horizontal gradient with adjustable stop positions

Horizontal gradient with adjustable stop positions.

![grad_1](screenshots/grad_1.png)

### grad_2 — Linear gradient with arbitrary start/end points

Linear gradient with arbitrary start/end points.

![grad_2](screenshots/grad_2.png)

### grad_3 — Radial gradient with focal point

Radial gradient with focal point.

![grad_3](screenshots/grad_3.png)

### grad_4 — Conical gradient

Conical gradient.

![grad_4](screenshots/grad_4.png)

## Snapshot

### snapshot_1 — Capture and display a rotated/scaled widget snapshot

Capture and display a rotated/scaled widget snapshot.

![snapshot_1](screenshots/snapshot_1.png)

## Implementation Coverage

Status of all [LVGL 9.5 examples](https://docs.lvgl.io/9.5/examples.html) in oxivgl.

**Legend:** Done = ported, Skip = intentionally skipped (reason noted), Missing = has wrapper but no example yet, No wrapper = widget not yet wrapped.

### Core Examples

| Category | LVGL | Done | Skip | Notes |
|---|---|---|---|---|
| Getting Started | 4 | 4 (+4 extra gradient examples) | 0 | |
| Styles | 21 | 20 | 1 | style19 (meta/benchmarking); +shared_styles1 extra |
| Animations | 5 | 5 | 0 | |
| Events | 5 | 5 (+1 extra trickle) | 0 | |
| Flex | 6 | 6 | 0 | |
| Grid | 6 | 6 | 0 | |
| Scroll | 9 | 9 | 0 | |
| Observer | 6 | 3 | 3 | observer4/5/6 blocked: animation/timer/style infrastructure not yet wrapped |
| Gradients | 4 | 4 | 0 | |
| Snapshot | 1 | 1 | 0 | |

### Widget Examples (wrapper exists)

| Widget | LVGL | Done | Missing | Notes |
|---|---|---|---|---|
| obj | 3 | 3 | 0 | |
| arc | 3 | 3 | 0 | |
| arclabel | 1 | 1 | 0 | v9.5 new widget |
| bar | 7 | 8 | 0 | widget_bar8 extra: bg_image recolor |
| button | 3 | 3 | 0 | |
| calendar | 2 | 2 | 0 | |
| checkbox | 2 | 2 | 0 | |
| dropdown | 3 | 3 | 0 | |
| image | 5 | 5 | 0 | |
| label | 7 | 7 | 0 | label7 simplified (no `lv_translation_*` in bindings) |
| led | 1 | 1 | 0 | |
| line | 1 | 1 | 0 | |
| list | 2 | 2 | 0 | |
| menu | 5 | 5 | 0 | |
| msgbox | 2 | 3 | 0 | msgbox3 uses blur API (no-op on SDL) |
| chart | 3 | 6 | 0 | chart6/7/8 new |
| buttonmatrix | 1 | 3 | 0 | btnmatrix2/3 new |
| keyboard | 2 | 2 | 0 | |
| roller | 3 | 3 | 0 | |
| scale | 12 | 12 | 0 | |
| slider | 4 | 4 | 0 | |
| switch | 2 | 2 | 0 | |
| textarea | 4 | 4 | 0 | |
| canvas | 12 | 11 | 1 | canvas_8 (vector graphics, ThorVG) |
| table | 2 | 2 | 0 | |
| tabview | 2 | 2 | 0 | |
| tileview | 1 | 1 | 0 | |
| span | 1 | 1 | 0 | |
| imagebutton | 1 | 1 | 0 | |
| animimg | 1 | 1 | 0 | |
| win | 1 | 1 | 0 | |
| lottie | 3 | 0 | 0 | Abandoned — ThorVG bloats firmware beyond flash limits (see below) |
| spinner | 1 | 1 | 0 | |
| spinbox | 1 | 1 | 0 | |

### Widgets Without Wrappers (new in v9.5)

| Widget | Notes |
|--------|-------|
| gif | GIF animation playback (no LVGL examples yet) |
| ime_pinyin | Chinese Pinyin input method (no LVGL examples yet) |

### Totals

| | Count |
|---|---|
| LVGL examples total | ~193 |
| oxivgl done | 171 |
| Verified on ESP32 (fire27) | 148/160 |
| Skipped (intentional) | 1 |
| New in v9.5 (not ported) | 2 |
| Missing (wrapper exists) | 1 |
| Abandoned (lottie) | 3 |
| No wrapper (v9.5 new widgets) | 3 |

## Running

```sh
# Interactive SDL2 window:
./run_host.sh getting_started1

# Headless screenshot (no window):
./run_host.sh -s getting_started1

# Screenshot all examples:
./run_host.sh -s

# Flash to ESP32:
./run_fire27.sh getting_started1
```
