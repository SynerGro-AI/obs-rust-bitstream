# Public OBS Rust Rewrite — Plan

Goal: Seraphina Studio, a desktop app for paint, composite, 3D,
animation, streaming and a materials lab, and the tested library
behind it.
**Work smarter, not harder.**

## Local compute first (design principle)

The owner's rule: "Use Python scripts whenever they are more efficient
than spending API credits, and let the CPU do the work instead of
burning credits. Prefer local compute over paid API calls wherever the
CPU can handle it faster or cheaper."

* Each spec has a "Paid calls still needed" list. For every paid call
  it says what the call is for and why the CPU can't do the job.
* Tests make no paid calls by default. Billed checks are `#[ignore]` and
  are run by hand, as `live_xai_*` and `live_imagine` already are.
* Offline tooling, such as fixtures, test values and doc checks, is a
  local Python script.

## Where things stand

* The library modules that exist are listed in README.md (the list is
  checked against `src/lib.rs` by a test). Each has unit tests.
* `bitstream` is a port of OBS `bitstream_reader` and keeps a C ABI.
* Recording and RTMP streaming run in the app through ffmpeg
  (`src/bin/ui/output.rs`), tested against a local ffmpeg RTMP listener
  and, on Linux CI, mediamtx. Their source is the canvas, the screen, or a
  webcam (`src/bin/ui/camera.rs`: V4L2 on Linux, Media Foundation on
  Windows; macOS AVFoundation is planned). Live output has an optional
  stream delay (`encoder_packet` + `output_delay_queue` between the
  encoder ffmpeg and a copy-only relay ffmpeg).
* `tools/obs_cpp_to_rust_parser.py` turns an OBS `.c` file into a Rust
  skeleton that documents the lessons in `tools/lessons.json`.

## Planned (not built)

The v1 cleanup removed modules that the app and examples never used
(the `every_library_module_is_reached_from_a_binary_or_example` test
keeps it that way). They come back only as real implementations, each
with a test. The tables in README.md ("Planned (not built)") hold the
target dates (America/Edmonton; targets, not promises):

* v1 cleanup and M4 part 2: Wed Oct 28, 2026 (target).
* First green CI run, run locally: Sun Nov 1, 2026 (target). The GitHub
  run waits for the billing lock to lift; it does not unlock by itself.
* v0.2.0 release: Fri Nov 6, 2026 (target, pending the owner's approval).

Everything below comes after v0.2.0 and has no date yet:

1. Seraphina.AGI as a separate local process (Grok is today's
   provider): proposed.
2. Remove `dist/` and root clutter: M5.
3. Physics foundation: M6.
4. OBS sources, scenes and the rest of Stream (E6): M7+.
5. The rest of the Photoshop / After Effects / Maya tools listed as
   `[ ]` in TOOLS.md: M7+.
6. Topic backgrounds (E7): its own milestone after v0.2.0. The date is
   pending (the Auditor's round-2 question 1 to the user).

## E7: Topic backgrounds (spec; planned, not built)

The owner asked for this (relayed): "Either grok imagine or the ai wired,
or if it is a weather topic, the website for the live weather map of the
area speaking about. (As an example) or sunny topic a photo of a beach
from somewhere unless specific". Nothing in this section is built. It is
a spec, revised after the Auditor's review (`spec-review-c4-bg.md`).
Weather and sunny are examples of one general rule: a topic maps to a
kind of source, and the background changes only when the topic calls for
it and a setting allows it.

**Milestone:** E7, its own milestone after v0.2.0 (it builds on Stream
work, E6). The date is pending: M4b has slipped, and the v0.2.0 date is
the Auditor's round-2 question 1 to the user (E7-R6). Placing E7 after
v0.2.0 is the Auditor's recommendation (BG-8), pending user confirmation. It depends on a compositor
layer that does not exist yet, so step 1 below builds that layer, with
its own tests, before anything else.

**Decided by the owner (Oct 24 2026: "Yes, go with all four
recommendations"):**

* OpenStreetMap tiles now, behind a switchable tile URL setting.
* Topics come from chat text first. Speech follows right after, through
  the $0.10/h REST speech-to-text endpoint.
* Every Grok Imagine background needs your approval for that image, and
  the default model is `grok-imagine-image` ($0.02 per image).

**Proposed by the Auditor, written in as the design, pending user
confirmation:**

* Offline GeoNames only, with no Nominatim.
* No CC BY-SA photos and no Pexels.
* An honest "no licensed live radar" error outside ECCC radar coverage.
* Camera picture-in-picture without a person cut-out is fine for v1.

### What exists today and what is missing

* **Real today:**
  * the chat text;
  * the Grok chat model with tool calling (`llm.rs`, `AppCommand` in
    `commands.rs`);
  * Grok Imagine (`videogen.rs::generate_image`), behind the **Run** card
    and the spend cap (`usage.rs`);
  * `ureq`, the `image` crate, and the OS credential store
    (`secrets.rs`).
* **Missing:**
  * a background layer: the output source is one of Canvas, Screen or
    Camera (`output.rs`), with no compositing;
  * speech-to-text: the microphone goes only into ffmpeg;
  * a geocoder and map code.

### Step 1: the compositor background layer

A new **Studio background** layer in the output path, with these layers
from bottom to top:

1. the background image, scaled to cover the output (1280×720 by
   default);
2. the camera picture as a picture-in-picture box (a corner and size you
   choose) when the source is Camera, or the canvas inset when the
   source is Canvas;
3. an attribution strip along the bottom, drawn into every output frame
   whenever the background came from a third party.

The Stream preview shows the same frame. Person segmentation is later.
Step 1 has its own tests (listed below) and ships with a plain colour
or an image you pick, before any topic logic.

### Settings (off by default)

* **Topic backgrounds:** Off (default), Ask me, or Automatic. Off means
  no suggestion tool, no fetch and no change.
* **Per-source switches** (all off): Live radar, Photos, Generated.
* **Photos** exist only if the owner picks a photo source (pending, see
  Photos). If they do, they always ask, with a preview, even in
  Automatic mode.
* **Generated images** always ask: the **Run** card shows the exact
  prompt, the model and the price for each image.
* **Background image model:** `grok-imagine-image` by default (its own
  setting, separate from `SERAPHINA_IMAGE_MODEL`).
* **Map tile URL:** default `https://tile.openstreetmap.org/{z}/{x}/{y}.png`,
  changeable in Settings without a software update.
* **Home location:** empty by default. Its help text says: "If a radar
  map of your home location appears on stream, viewers can see roughly
  where you are. The map area is also sent to Environment and Climate
  Change Canada and to the tile server."
* **Pin background:** holds the current background until you unpin it.

### Suggesting is not applying: two separate names

* **`suggest_studio_background(topic, kind, place, confidence)`** is the
  AI's only tool, offered on an existing chat turn only when the local
  matcher can't settle the topic. It is a suggestion: it feeds the threshold,
  hysteresis and cooldown logic and changes nothing by itself. `kind` is
  `live_weather`, `photo`, `generated` or `none`. `confidence` is the
  model's own number. It is uncalibrated, so the README makes no
  accuracy claim, and the tests check the gating logic only. With the
  setting off, the tool is not offered.
* **`set_studio_background(asset)`** is an `AppCommand` that only the UI
  and the background pipeline dispatch. Like `set_stream_key`, it is not
  offered to the AI. It takes only an asset that has already been
  fetched, licence-checked and cached, so applying it is synchronous and
  one undo step. A failed fetch never reaches it.

### Topic detection: local first, the model only when it's unclear

**Local first** (no model, no network, no cost):

* Every chat message, and every speech transcript window, first goes
  through a local matcher.
* **Places:** the offline GeoNames lookup (see Location) finds the place
  names.
* **Topics:** a small table that ships with the app maps clear words to
  a kind. For example, "weather", "rain", "snow", "storm" and "radar"
  map to `live_weather`, and "sunny", "beach" and "summer" map to
  `photo` (beach).
* **A clear case** is exactly one topic-table kind and at most one place
  that resolves without ambiguity. A clear case becomes a suggestion at
  once, with confidence 1.0, and still goes through the agreement,
  hysteresis and cooldown rules below.
* The table is a constant in one file, and tests check it.

**The model only when it's ambiguous, and only on a call you are
already making:**

* For chat text, the chat model already answering the turn is offered
  `suggest_studio_background`, but only when the local matcher found a
  possible topic it couldn't settle. That is two kinds in the table, an
  ambiguous place, or a topic word with no table entry.
* The tool rides on that existing turn. Background detection never makes
  a separate model call.
* For speech, there is no chat turn to ride on, so speech uses the local
  matcher only. An unclear speech window changes nothing. In Ask me mode
  it can show a card with the candidates for you to pick. There is no
  per-window classification call.

**Speech (right after chat, decided):** the REST endpoint `POST
https://api.x.ai/v1/stt` ($0.10 per hour of audio, model
`grok-voice-transcribe-2.0`; prices checked on docs.x.ai). It does not
use the $0.20/h websocket stream.

* A second ffmpeg reads the same microphone as the output settings
  (`dshow` on Windows, `pulse` on Linux) and writes 30 s WAV chunks
  (16 kHz mono).
* Each chunk is sent as multipart over the existing `ureq`, so no new
  crate is needed.
* The cost shown before you turn speech on is the audio only, $0.10 per
  hour. There are no extra classification calls.
* It is opt-in, shows "Transcribing your mic" while it runs, and is
  billed through the existing meter. No audio is stored, and the
  transcript is kept in memory for 2 minutes only.
* A 30 s delay is well inside the 120 s cooldown.

**Without an xAI key,** chat-text topics still work for the clear cases
through the local matcher. Unclear cases and speech are off, and the
settings say why.

### When the background changes

* **Threshold:** a model suggestion counts only if `confidence ≥ 0.75`.
  A clear local match counts as 1.0.
* **Agreement:** the same `kind` and resolved place in two consecutive
  suggestions.
* **Hysteresis:** the new topic must beat the current one's confidence by
  0.15. Going back to the default needs 3 windows in a row with `none`.
* **Cooldown:** at least 120 s between automatic changes, and at least
  60 s on screen.
* **Ask me:** a card ("Switch background to: live radar, precipitation type,
  Edmonton, AB?") instead of a change. "Not now" mutes that topic for
  10 minutes.
* All the numbers are constants in one place, and tests check them.

### Location (offline GeoNames only, pending confirmation)

* The model returns the place as it was said, and the app resolves it.
  The model never picks coordinates.
* The app ships the GeoNames `cities15000` extract (about 25,000 places,
  CC BY 4.0, credited in About). There is no network lookup and no rate
  limit.
* **Ambiguous:** if two or more places match and the largest is under 10
  times the next one's population, the app asks ("Which Paris? Paris,
  France · Paris, Texas, US · Paris, Ontario, CA"). With one clear
  winner, the resolved name is shown on the card and in the strip.
* **Unknown** (for example a town under 15,000 people): an error, plus a
  list of the nearest listed cities to pick from. The background stays
  as it is.
* **No place named:** live topics use the home location, or ask if it is
  empty. Photo topics (only if photos ship) use the topic's general
  entry, a beach.
* Nominatim is not used. Its policy caps the whole app's users at 1
  request per second, treats periodic app requests as bulk geocoding,
  licenses its data under ODbL, and has a clause on LLM-built software.

### Live weather radar: ECCC only

**Source:** MSC GeoMet WMS (`https://geo.weather.gc.ca/geomet`).
NOAA/NWS is cut from v1: the ECCC mosaic already includes the US radars.

**One layer, no season rule** (Auditor round 2, E7-R2). The app draws
`Radar_1km_SfcPrecipType` ("Radar surface precipitation type"). The
GeoMet radar page lists it as the North American 1 km surface
precipitation type composite, with its coverage layer
`Radar-Coverage_SfcPrecipType`, updated every 6 minutes with the last 3
hours available
(<https://eccc-msc.github.io/open-data/msc-data/obs_radar/readme_radar_geomet_en/>,
read Oct 24 2026).

* The radar itself says rain, snow or mixed. The Auditor's live
  GetCapabilities check (12:30 PM MT, Oct 24 2026) found the time step
  `PT6M`, the styles `SfcPrecipType_Dis` and `SfcPrecipType_Dis_Fr`, and
  an abstract saying the layer "can distinguish occurrence and type of
  precipitation and can recognize several levels of precipitation
  intensity". A live `GetFeatureInfo` (text/plain) returned a classed
  value (`310`, `Light Rain`). A point with no precipitation returned
  "Search returned no results".
* **The topic word decides only whether radar is shown** (any
  `live_weather` word in the topic table). It never picks a layer. There
  is no snow-word list, no rain/snow switch and no season constant.
* The legend comes from `GetLegendGraphic` with style `SfcPrecipType_Dis`
  and is drawn on screen with the label "Precipitation type (radar)".
  `SfcPrecipType_Dis_Fr` is for a French UI, which does not exist today.
* The rain and snow rate layers (`RADAR_1KM_RRAI`, `RADAR_1KM_RSNO`) and
  their coverage layers are not used. `RADAR_1KM_RDBR` does not exist and
  is not used.
* Time dimension: PT6M steps over the last 3 hours, defaulting to the
  latest.
* The bounding box is read from this layer's own capabilities fixture.
  The rate composite's box (`RADAR_1KM_RRAI`, GeoMet-Weather 2.40.4,
  Oct 24 2026) was longitude −170.32 to −50.00, latitude 16.93 to 67.19.
  The `Radar_1km_SfcPrecipType` box is not recorded yet.

**Coverage: one outline per time step, tested locally** (E7-R3):

* Actual coverage changes over time, and `Radar-Coverage_SfcPrecipType`
  ("dynamic radar coverage", every 6 minutes) shows it.
* The Auditor measured that `GetFeatureInfo` (JSON) on the coverage layer
  `RADAR_COVERAGE_RSNO` returns the whole North American coverage
  MultiPolygon, whatever the point asked about. That is tens of KB as seen
  through a web fetch; the byte size was not measured.
* So the app fetches the `Radar-Coverage_SfcPrecipType` outline **once
  per time step**, caches it keyed by that step's valid time, and tests
  the place's point against it locally (point-in-polygon on the CPU).
  Another place, or another redraw in the same step, makes no new
  coverage request.
* Whether `Radar-Coverage_SfcPrecipType` answers the same way as
  `RADAR_COVERAGE_RSNO` is not verified yet. The captured fixture (below)
  settles it. If it returns only a local feature, this spec is revised
  before the code is written.
* **Outside coverage** (outside the layer's box, or outside the outline),
  the app shows the error "No licensed live radar covers {place}" and
  keeps the current background (pending confirmation).

**The point mapping is pinned by a recorded fixture** (E7-R3). One
EPSG:4326 query near (−95, 50) returned a feature at (−148.36, 60.19)
(Auditor, round 2). Two explanations fit and neither is verified: WMS
1.3.0 puts latitude first for EPSG:4326, and a coverage response's
feature is the whole outline, not the point. Before any point query is
coded, the capture below records the same point (−95, 50) on
`Radar_1km_SfcPrecipType` as EPSG:4326 (latitude first), CRS:84
(longitude first) and EPSG:3857. The test
`geomet_point_query_fixture_pins_axis_order` checks which request form
answers for the asked point. The app uses only the form that fixture
proves. Point queries made at run time are not in the 721-a-day budget
below (it counts only the per-step refresh and the legend). Each one
would raise it, so the budget and `tools/geomet_budget.py` are revised
when point queries are coded.

**"Live" follows the radar's own time:**

* The valid time comes from the layer's time dimension and is drawn as
  "Radar HH:MM" in local time.
* "Live" is shown only when that time is no more than 18 minutes (3
  steps) older than the HTTP `Date` header of the same response, which
  avoids trusting the local clock. Otherwise the strip says "Not live:
  radar from HH:MM", even after a successful fetch, because a 200
  response can carry an old default time during an ECCC outage.
* The map refreshes every 6 minutes, on the step, and only while the
  radar background is visible (see the MSC usage policy below). If a
  refresh fails, the image stays with the "Not live" label and the
  reason.

**Credit, as the ECCC licence words it** (Data Services End-use Licence
v2.1.1, Aug 2026, s.3). The composite mixes Canadian and NOAA radars, so
the strip uses the licence's multiple-originator form:

> Data Source: Environment and Climate Change Canada and NOAA (US radars)

* It links to the licence,
  <https://eccc-msc.github.io/open-data/licence/readme_en/>, in About and
  in the suggested stream description.
* There are no ECCC names used as branding, no crests and no logos (s.4),
  and nothing that suggests endorsement (s.5). The `LogoURL` in
  GetCapabilities is ignored.
* The wording is kept as written (Auditor round 2, 2a). The "NOAA (US
  radars)" part rests on the `RADAR_1KM_RRAI` abstract. When the
  `Radar_1km_SfcPrecipType` capabilities fixture is captured, its
  abstract is checked for the US radars too. If it doesn't name them,
  the wording goes back to the owner rather than being changed quietly.

**MSC usage policy for GeoMet requests** (E7-R1). This is separate from
the licence: the MSC Open Data Services Usage Policy,
<https://eccc-msc.github.io/open-data/usage-policy/readme_en/>, read Oct
24 2026. The radar usage tips are on the GeoMet radar page linked above.

* **A meaningful User-Agent** on every GeoMet request, the same one as
  the tiles: `SeraphinaStudio/<version>
  (+https://github.com/SynerGro-AI/seraphina-studio)`. The policy says:
  "Add a meaningful HTTP User-Agent header to HTTP requests".
* **No `no-cache` headers:** never `Cache-Control: no-cache` or `Pragma:
  no-cache`, as the policy asks.
* **GetCapabilities is always layer-filtered**, with
  `&layer=Radar_1km_SfcPrecipType` or `&layer=Radar-Coverage_SfcPrecipType`.
  That is the vendor extension the policy asks for to get smaller XML.
  The full capabilities document is never requested.
* **No bulk or tile-pyramid retrieval:** one GetMap per step for the box
  on screen, never WMS tiles in bulk and never a local archive. The
  policy says "Bulk and batch retrieval of WMS tiles is prohibited", and
  a local archive belongs on MSC Datamart, which the app does not use.
* **Only while visible:** the radar refreshes only while its background
  is in the output being recorded or streamed, or in a Stream preview
  that is on screen. It pauses when the background is hidden or
  replaced, and when the window is minimised with nothing recording or
  streaming. **The minimised-but-recording exception is pending user
  confirmation** (open question 4). The Auditor wrote "pause when hidden
  or minimised". The exception (a minimised window that is still
  recording or streaming keeps refreshing, because the radar is still on
  viewers' screens) was added in round 2 and has not been approved.
  Until it is, the strict rule is the fallback: pause whenever the
  window is minimised.
* **The per-install budget**, computed by `tools/geomet_budget.py`:
  * one time step is 6 minutes, so there are 240 steps a day;
  * at most 3 GeoMet requests per step: the layer-filtered
    GetCapabilities (to learn the newest valid time), one GetMap, and
    one coverage outline;
  * the legend PNG at most once a day (one style, cached on disk);
  * **so the budget is 721 requests per install per day** (3 × 240 + 1).
    A counter kept per UTC day enforces it. At the limit the app stops
    fetching, and the strip says "Not live: daily radar request budget
    reached".
  * GeoMet requests are counted per session and the count is written to
    the log when the session ends, so the logs can be reviewed as the
    policy asks.
* **When MSC asks to be contacted:** the policy says to contact MSC when
  an application makes 86,400 requests a day or more (about 1 a second).
  For the whole app that is:
  * about 119 installs showing radar around the clock at the full budget
    (86,400 ÷ 721);
  * 180 installs if only GetMap plus one other call per step were counted
    (86,400 ÷ 480), and 360 for GetMap alone (86,400 ÷ 240). That is the
    Auditor's 180–360 range, which leaves out the third call per step;
  * about 714 installs if radar is visible 4 hours a day (40 steps × 3 +
    1 = 121 requests).
* **Owner action (noted, not done):** before a wide public release, the
  owner contacts MSC about the expected volume. The owner also subscribes
  to the GeoMet-Info announcement list, which the policy asks users to
  follow so layer changes are seen in time.

**Base map: OpenStreetMap tiles (decided), following the Tile Usage
Policy:**

* the exact URL `https://tile.openstreetmap.org/{z}/{x}/{y}.png` by
  default, switchable in Settings;
* a unique User-Agent (`SeraphinaStudio/<version>
  (+https://github.com/SynerGro-AI/seraphina-studio)`);
* the HTTP cache headers honoured, with conditional requests
  (`If-None-Match` / `If-Modified-Since`), and 7 days only when a tile
  carries no cache headers;
* no prefetch, no bulk download and no offline use: only the tiles for
  the box on screen, at zoom 10 or less;
* "© OpenStreetMap contributors" in the strip, linked to
  <https://www.openstreetmap.org/copyright> in About, plus a "Report a
  map issue" link to <https://www.openstreetmap.org/fixthemap>;
* the spec records the policy's warning: "Commercial services, or those
  that seek donations, should be especially aware that access may be
  withdrawn at any point." That is why the tile URL can be switched.

### Photos: source pending your decision (nothing about photos is built)

The round-1 design does not work. It searched curated Commons
categories at runtime, used a category blocklist, and refused files that
depict a human (Q5). The Auditor measured it with `beach_audit.py`
(round 2) on "Featured pictures of beaches" and "Quality images of
beaches": 258 unique files ("Valued images of beaches" does not exist).

* 47 files are licence-eligible (public domain, CC0 or CC BY, no SA).
  CC BY-SA 4.0: 175; CC BY-SA 3.0: 33.
* 0 of those 47 have a "depicts" (P180) statement at all, so the Q5 check
  filters nothing.
* 4 eligible files are in people categories despite curation. One
  curated file is in "Nude men in Canada"; it was excluded only because
  it is CC BY-SA.
* The people and nudity categories are large and nested, so a blocklist
  can never be complete.

So the photo source is **pending user decision** (the Auditor's round-2
question 3). These are the two options from the report. This spec picks
neither, and photos stay off until the owner decides.

**(A) A pinned set you review once.**

* A local script (like `beach_audit.py`) builds candidates per topic: the
  licence filter, landscape at least 1280 px wide, and the people/nudity
  category regex as a backstop. The Auditor's run found about 39 clean
  beach candidates, for example "Beach Montechiaro.jpg" (CC0) and "Beach
  of Cape Fiolent, Crimea.jpg" (CC BY 4.0).
* You review the thumbnails once.
* The app ships, per photo: title, pageid, sha1, author, licence and the
  credit line.
* At runtime it fetches only pinned files and checks the sha1. On a
  mismatch the photo is refused, the current background stays, and the
  reason is shown.
* The Q5 check is dropped. Topics outside the set get the map/radar, or
  Imagine with approval.
* Tests: `pinned_photo_sha1_mismatch_is_refused`, and
  `pin_builder_refuses_sa_and_people_categories` (run against a recorded
  API fixture).

**(B) No photos.** Topics get the map/radar, or a Grok Imagine image with
approval. The Photos switch, the Commons fetch and the photo tests are
removed.

**If photos ship (option A), these rules hold:**

* **Licences:** public domain, CC0, and CC BY 2.0, 2.5, 3.0 and 4.0
  only. CC BY-SA, unknown licences and anything else are refused (no
  CC BY-SA and no Pexels, pending confirmation).
* **Preview before showing:** always a card with the photo, its credit
  and [Use] [Skip], even in Automatic mode.
* **A fixed credit line**, drawn into the frame:

  > Photo: {author} / Wikimedia Commons / {licence} / scaled and cropped

  * `{author}` comes from `extmetadata.Artist`, with the HTML stripped
    and the length capped at 60 characters.
  * The licence URL and file page are in About and in the suggested
    stream description.
  * For CC BY 2.0 and 2.5, any copyright notice in the file's
    `Attribution` metadata is kept.
* **Requests:** the API is called with a User-Agent that names the app
  and gives contact details (Wikimedia's User-Agent policy).
* Unsplash and Pexels are not used. Their guidelines restrict
  wallpaper-style apps, and Unsplash requires non-automated use and
  server-side keys.

### Generated images: Grok Imagine (approval for every image, decided)

* **Pinned request:** model `grok-imagine-image`, `n: 1`, `aspect_ratio:
  "16:9"`, `resolution: "1k"`, and `response_format: "b64_json"`. 1K is
  enough for a 720p backdrop. `quality` is a `grok-imagine-image-2.0`
  parameter only. If you switch the background model to 2.0, the request
  also pins `quality: "low"`.
* **Price, as the code charges it today** (`usage.rs::image_price`; the
  `generate_image` call itself sends no size or quality yet):
  * `grok-imagine-image` gives $0.02, which matches xAI's price of $0.02
    at 1K or 2K.
  * Any model name containing "2.0" gives $0.08, which errs high. xAI
    charges $0.04 at 1K low and $0.08 at 2K medium.
  * When the response carries the real cost (`cost_in_usd_ticks`), the
    meter uses that instead.
  * The E7 code makes the estimate match the pinned request ($0.04 for
    2.0 at 1K low), and a test checks that the Run card shows the table
    price.
* **The prompt** is built from a template: the topic (at most 60
  characters), plus the resolved GeoNames place name, plus a fixed style
  line ("wide studio backdrop, no text, no people").
  * The topic is free text from the model, so the app cannot guarantee
    that it holds no names. That is why the exact prompt is always shown
    on the Run card before you approve.
  * No chat history is sent.
* **Limits:** the spend cap still blocks the call with a toast. A
  generated image is cached per prompt for the session, so returning to
  a topic does not bill again.
* "Generated image (Grok Imagine)" is drawn in the strip. Later, the
  wired provider (Seraphina.AGI as a local process, after v0.2.0) is a
  second generator with the same rules.

### Rendering a live map into the scene

egui has no browser engine. The app draws the map from the same data
the radar websites use:

1. one WMS `GetMap` request (EPSG:3857, transparent PNG, a box about
   400 km wide, at the output size);
2. the matching OSM tiles, stitched with `image`;
3. the radar drawn over the base map, then a place marker, the place
   name, the layer label, the legend and the strip;
4. the result uploaded as an egui texture and handed to the compositor
   as RGBA. No new crate is needed.

Embedding a real web page is later. wry's child webview can show a page
inside the window, but wry has no released frame-capture API
(screenshot PR #1674 is open), so the page can't go into stream frames.
Capturing a browser window waits for the Window source (M7+), with a
per-site terms check.

### Failure: visible, and nothing passed off as live

* Every failure shows a toast and a status line with the real reason.
  Failures include:
  * no network, an HTTP error, a 10 s timeout or a decode error;
  * a photo whose sha1 doesn't match its pin (option A only);
  * a place that is unknown, ambiguous or outside coverage;
  * the spend cap, or the setting being off.
* **The current background stays.** There is no silent fallback to
  another source, no stock image, and no old map labelled as current. A
  photo or a generated image is never labelled live.
* A failed attempt counts toward the cooldown, with one retry after 60
  s.

### Local compute first, and the paid calls still needed

**The order of sources:** a real map or photo always comes before a
generated one.

1. **Live radar and the map:** free ECCC and OSM fetches. Stitching,
   drawing and compositing are done locally with the `image` crate.
2. **A pinned Commons photo, only if the owner picks option A**
   (pending): a free fetch, then the sha1 check, local scaling, cropping
   and drawing of the credit.
3. **Grok Imagine:** only when no licensed real image fits, or when you
   pick Generated yourself. Each image needs your approval on the Run
   card.

**Done on your own CPU:**

* the topic and place matching;
* the GeoNames lookup;
* the gating rules;
* the radar time check, and the coverage point-in-polygon test against
  the cached outline;
* the GeoMet request budget and the per-session request count;
* tile stitching, compositing and the attribution strip;
* if option A is picked: the pin builder's licence and category filters
  (a local script), the sha1 check and the credit line.

**Tests:** no test makes a paid call by default. The only billed test,
`xai_stt_live_transcribes_a_recorded_wav`, is `#[ignore]` and run by
hand. The other network tests are free (GeoMet, OSM, and Commons if
option A is picked) and are also skipped by default.

**Paid calls still needed:**

| Paid call | What it is for | Why the CPU can't do it (today) |
|---|---|---|
| xAI speech-to-text, `POST /v1/stt`, $0.10 per hour of audio | Turning your mic audio into text, so spoken topics can be matched locally | The app ships no local speech model. A local engine (for example whisper.cpp) could do this on a fast enough PC, at the cost of a model download of hundreds of MB and CPU or GPU time during a stream. It is worth measuring later; the owner chose REST speech for now. |
| Grok Imagine, `grok-imagine-image`, $0.02 per image | A background image when no licensed real photo or map fits | Making a new picture needs a large image model and usually a GPU, which the target PCs may not have. It is the last resort, and every image needs approval. |
| The chat turn you already make (no new call) | Settling unclear topics with `suggest_studio_background` | Understanding free text that the topic table doesn't cover needs a language model. It rides on the existing turn and adds only the tool's schema and arguments, a few hundred tokens. |

GeoMet, the OSM tiles and Commons (option A only) are free network
calls, not paid ones.

### Keys

* v1 sources need no key (GeoMet, OSM tiles, Commons if option A is
  picked, offline GeoNames). Imagine and speech-to-text use the existing xAI key from the
  OS credential store (`Secret::XaiKey`) or the environment.
* Any later key (a commercial tile provider) becomes a new `Secret` in
  `secrets.rs`. It is never stored in the project file, the settings, the
  chat, the prompt, logs or crash reports, and never passed as an AI
  tool argument.

### Tests (to write with the code; real fetches or recorded real responses, no fakes)

**Recorded fixtures** are real responses saved from the real services,
with the request URL and the fetch time beside them in
`tests/fixtures/background/`.

* This box cannot reach `geo.weather.gc.ca` with `curl`: the connection
  times out or the TLS handshake ends early. A web fetch tool can reach
  it, and the Auditor saw the same.
* **So the GeoMet fixtures must be captured on the owner's Windows PC.**
  Until then, those tests don't exist, and nothing is faked to stand in
  for them.

**Capture commands.** Type or paste them at a Command Prompt (cmd.exe,
not PowerShell and not a `.bat` file) on Windows 10 or 11, which ships
`curl.exe`, from the repository folder. First run:

```
mkdir tests\fixtures\background\geomet
cd tests\fixtures\background\geomet
```

Then run each line below. Each one saves the response, its headers (the
HTTP `Date` header is the fetch time) in `<name>.headers.txt`, and
appends the request URL to `requests.txt`:

```
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D caps_precip.headers.txt -o caps_precip.xml -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetCapabilities&layer=Radar_1km_SfcPrecipType" >> requests.txt
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D caps_coverage.headers.txt -o caps_coverage.xml -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetCapabilities&layer=Radar-Coverage_SfcPrecipType" >> requests.txt
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D getmap_precip_edmonton_3857.headers.txt -o getmap_precip_edmonton_3857.png -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetMap&layers=Radar_1km_SfcPrecipType&styles=SfcPrecipType_Dis&crs=EPSG:3857&bbox=-12970673,6895320,-12297471,7273995&width=1280&height=720&format=image/png&transparent=true" >> requests.txt
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D legend_precip_dis.headers.txt -o legend_precip_dis.png -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetLegendGraphic&sld_version=1.1.0&layer=Radar_1km_SfcPrecipType&style=SfcPrecipType_Dis&format=image/png" >> requests.txt
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D coverage_outline_3857.headers.txt -o coverage_outline_3857.json -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetFeatureInfo&width=101&height=101&i=50&j=50&layers=Radar-Coverage_SfcPrecipType&query_layers=Radar-Coverage_SfcPrecipType&info_format=application/json&crs=EPSG:3857&bbox=-12634577,7084152,-12633567,7085162" >> requests.txt
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D point_edmonton_3857.headers.txt -o point_edmonton_3857.txt -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetFeatureInfo&width=101&height=101&i=50&j=50&layers=Radar_1km_SfcPrecipType&query_layers=Radar_1km_SfcPrecipType&info_format=text/plain&crs=EPSG:3857&bbox=-12634577,7084152,-12633567,7085162" >> requests.txt
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D point_odd_epsg4326.headers.txt -o point_odd_epsg4326.json -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetFeatureInfo&width=101&height=101&i=50&j=50&layers=Radar_1km_SfcPrecipType&query_layers=Radar_1km_SfcPrecipType&info_format=application/json&crs=EPSG:4326&bbox=49.99,-95.01,50.01,-94.99" >> requests.txt
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D point_odd_crs84.headers.txt -o point_odd_crs84.json -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetFeatureInfo&width=101&height=101&i=50&j=50&layers=Radar_1km_SfcPrecipType&query_layers=Radar_1km_SfcPrecipType&info_format=application/json&crs=CRS:84&bbox=-95.01,49.99,-94.99,50.01" >> requests.txt
curl.exe -sS -A "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)" -D point_odd_3857.headers.txt -o point_odd_3857.json -w "%{url_effective}\n" "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0&request=GetFeatureInfo&width=101&height=101&i=50&j=50&layers=Radar_1km_SfcPrecipType&query_layers=Radar_1km_SfcPrecipType&info_format=application/json&crs=EPSG:3857&bbox=-10575857,6445771,-10574847,6446781" >> requests.txt
```

* What the lines capture: the layer-filtered capabilities for
  `Radar_1km_SfcPrecipType` and `Radar-Coverage_SfcPrecipType`; a
  `Radar_1km_SfcPrecipType` GetMap (EPSG:3857, transparent PNG, a box
  about 400 km wide around Edmonton, 1280×720); the `SfcPrecipType_Dis`
  legend; the coverage outline; a text/plain point query at Edmonton;
  and the three point-mapping queries at (−95, 50).
* `python3 tools/geomet_budget.py --check` fails if these lines and the
  budget numbers in this file drift from the script that computes them.
* The lines have not been run on Windows: this box has no Windows and
  can't reach the service. If a line fails, the error is recorded and
  the line is fixed here, not worked around.

**Step 1, the compositor:**

* `background_covers_the_output_frame`
* `camera_pip_sits_where_settings_say`
* `attribution_strip_is_drawn_into_every_output_frame`
* `preview_and_output_frames_match`

**Gating and commands:**

* `ai_gets_only_the_suggest_tool`
* `set_studio_background_is_ui_only_and_one_undo_step`
* `setting_off_means_no_tool_no_fetch_no_change`: a real local HTTP
  listener counts zero requests.
* `low_confidence_suggestion_changes_nothing`
* `two_agreeing_suggestions_are_needed_to_change`
* `hysteresis_keeps_the_background_on_a_brief_mention`
* `cooldown_blocks_a_second_change_within_120_s`

**Location:**

* `gazetteer_resolves_edmonton_to_alberta`
* `ambiguous_place_asks_instead_of_guessing`
* `unknown_place_offers_nearest_cities_and_keeps_the_background`
* `no_place_uses_home_location_or_asks`

**Radar (recorded fixtures):**

* `geomet_capabilities_fixture_gives_layers_time_and_bbox`
* `geomet_getmap_fixture_decodes_to_a_radar_image`
* `fresh_fetch_with_old_valid_time_is_not_live`
* `valid_time_within_18_min_of_server_date_is_live`
* `precip_type_legend_fixture_maps_classes`: the recorded legend and
  point query map class codes to labels (for example `310`, Light
  Rain).
* `coverage_outline_is_fetched_once_per_step`: a real local listener
  counts one coverage request for two places in the same step.
* `point_in_coverage_outline_fixture`: Edmonton is inside the recorded
  outline and a mid-Pacific point is outside, computed locally.
* `geomet_point_query_fixture_pins_axis_order`
* `place_outside_coverage_gives_the_no_radar_error`
* `eccc_credit_line_matches_the_licence_wording`

**MSC usage policy** (a real local listener on 127.0.0.1):

* `geomet_requests_send_user_agent_and_no_no_cache_headers`
* `geomet_capabilities_requests_are_layer_filtered`
* `radar_refresh_pauses_when_not_visible`
* `geomet_daily_budget_stops_at_721_requests`
* `geomet_session_request_count_is_logged`

**Tiles and failure** (a real local listener on 127.0.0.1 and recorded
real responses):

* `osm_tiles_send_the_user_agent_and_conditional_requests`
* `tile_url_setting_is_used`
* `fetch_failure_keeps_the_current_background_and_shows_the_reason`: a
  real HTTP 503.
* `photos_and_generated_images_are_never_labelled_live`

**Photos, only if option A is picked** (pending): the round-1 tests
`commons_search_fixture_parses_author_licence_and_url`,
`commons_filter_refuses_sa_unknown_blocklisted_and_people` and
`commons_choice_is_deterministic` are dropped. Option A's tests are:

* `pinned_photo_sha1_mismatch_is_refused`
* `pin_builder_refuses_sa_and_people_categories` (a recorded API
  fixture)
* `commons_credit_strips_html_and_caps_length`
* `photo_always_needs_preview_even_in_automatic`

**Imagine and keys:**

* `imagine_request_pins_model_ratio_and_resolution`
* `run_card_shows_the_usage_table_price_and_exact_prompt`
* `imagine_respects_the_spend_cap`
* `generated_background_is_cached_per_prompt`
* `keys_never_reach_project_settings_chat_or_logs`

**Local first:**

* `clear_topic_is_matched_locally_without_the_model`
* `suggest_tool_is_offered_only_when_the_local_match_is_unclear`
* `background_detection_never_makes_its_own_model_call`: a real local
  HTTP listener standing in as the API base counts exactly one request
  per chat turn.
* `speech_windows_make_no_classification_call`
* `real_sources_are_tried_before_imagine`

**Speech:** `stt_fixture_transcript_feeds_topic_detection` (a recorded
real response from xAI's speech-to-text endpoint) and
`stt_chunk_is_a_valid_30_s_wav`.

**Network tests** are `#[ignore]`, skipped by default, and must be listed
in README's Tests section as skipped by default:

* `geomet_live_getmap_returns_a_png_for_edmonton`
* `geomet_live_valid_time_is_recent`
* `osm_live_tile_returns_200_with_cache_headers`
* `commons_live_pinned_file_sha1_matches` (only if option A is picked)
* `xai_stt_live_transcribes_a_recorded_wav` (billed, run by hand)

A CI job that runs them on a schedule (`cargo test -- --ignored
<names>`) is **planned**. It can't run today: the organisation's GitHub
Actions are billing-locked. Until then, they are run by hand on a
machine with network, and the result is recorded in the PR.

### Open questions (still open)

The owner answered the four questions about money and schedule (see
"Decided" above). The Auditor's answers to the others are written in as
pending confirmation. Still to settle:

1. The photo source: (A) a pinned set you review once, or (B) no photos
   (the Auditor's round-2 question 3; see Photos). The Commons blocklist
   question is gone, because neither option uses a blocklist.
2. The E7 milestone date (the Auditor's round-2 question 1).
3. Whether the `Radar_1km_SfcPrecipType` abstract names the US radars,
   which the credit wording relies on. This is checked when the fixtures
   are captured.
4. Whether the radar keeps refreshing while the window is minimised but
   still recording or streaming (see "Only while visible" under the MSC
   usage policy). Pending user confirmation. Until answered, the radar
   pauses whenever the window is minimised, as the Auditor wrote.

The rain/snow season constants are gone: the precipitation type layer
replaced them (Auditor round 2, decision 12).

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve the public C ABI where a module ports OBS C code
- Fix the bugs in the safe path
- Delete before you add: no placeholder modules
- Every README claim is backed by a test or a measurement
- Local compute first: the CPU and local Python scripts before paid API calls

## How to contribute
Clone, run `cargo test --lib` and `cargo test --bin ui`,
open a PR.
