# Public OBS Rust Rewrite — Plan

Goal: Seraphina Studio, a desktop app for paint, composite, 3D,
animation, streaming and a materials lab, and the tested library
behind it.
**Work smarter, not harder.**

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
6. Topic backgrounds (E7): its own milestone after v0.2.0, dated when
   M4b merges.

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
work, E6). It gets a date once M4b merges. It depends on a compositor
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
* **Photos** always ask, with a preview, even in Automatic mode, until a
  stronger content filter exists.
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
  AI's only tool. It is a suggestion: it feeds the threshold,
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

### Topic detection

* **Chat text (first):** the model that already answers the chat turn
  may call `suggest_studio_background`, so there is no extra call.
* **Speech (right after, decided):** the REST endpoint `POST
  https://api.x.ai/v1/stt` ($0.10 per hour of audio, model
  `grok-voice-transcribe-2.0`; prices checked on docs.x.ai). It does
  not use the $0.20/h websocket stream.
  * A second ffmpeg reads the same microphone as the output settings
    (`dshow` on Windows, `pulse` on Linux) and writes 30 s WAV chunks
    (16 kHz mono).
  * Each chunk is sent as multipart over the existing `ureq`, so no new
    crate is needed.
  * Each 30 s of transcript gets one small classification chat call that
    returns the same fields as the tool. That is about 120 calls per
    hour, and the cost shown before you turn speech on includes both the
    audio and those calls.
  * It is opt-in, shows "Transcribing your mic" while it runs, and is
    billed through the existing meter. No audio is stored, and the
    transcript is kept in memory for 2 minutes only.
  * A 30 s delay is well inside the 120 s cooldown.
* There are no keyword lists. If no model is configured, the feature
  says so and stays off.

### When the background changes

* **Threshold:** a suggestion counts only if `confidence ≥ 0.75`.
* **Agreement:** the same `kind` and resolved place in two consecutive
  suggestions.
* **Hysteresis:** the new topic must beat the current one's confidence by
  0.15. Going back to the default needs 3 windows in a row with `none`.
* **Cooldown:** at least 120 s between automatic changes, and at least
  60 s on screen.
* **Ask me:** a card ("Switch background to: live radar, rain rate,
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
  empty. Photo topics use a general search ("beach").
* Nominatim is not used. Its policy caps the whole app's users at 1
  request per second, treats periodic app requests as bulk geocoding,
  licenses its data under ODbL, and has a clause on LLM-built software.

### Live weather radar: ECCC only

**Source:** MSC GeoMet WMS (`https://geo.weather.gc.ca/geomet`).
NOAA/NWS is cut from v1: the ECCC mosaic already includes the US radars.

**What it covers** (checked in GetCapabilities, GeoMet-Weather 2.40.4,
Oct 24 2026):

* `RADAR_1KM_RRAI` ("Radar precipitation rate for rain [mm/h]") and
  `RADAR_1KM_RSNO` ("… for snow [cm/h]") form the North American
  composite at 1 km. Its abstract says it "includes all the Canadian and
  American radars available in the network and which can reach a
  maximum of 180 contributing radars", and that "for the US Nexrad
  radars, ECCC uses the most similar product from the US Meteorological
  Service (NOAA)".
* Bounding box: longitude −170.32 to −50.00, latitude 16.93 to 67.19.
  The US–Canada border region and the US are therefore inside the
  mosaic, but within that box only areas in range of a working radar
  have data.
* **Actual coverage changes over time**, and the layers
  `RADAR_COVERAGE_RRAI` and `RADAR_COVERAGE_RSNO` ("Dynamic radar
  coverage", every 6 minutes) show it. Before drawing, the app asks the
  coverage layer for the chosen type (`GetFeatureInfo`, JSON) at the
  place's point.
* **Outside coverage** (outside the box, or a gap in the coverage
  layer), the app shows the error "No licensed live radar covers
  {place}" and keeps the current background (pending confirmation).
* Time dimension: PT6M steps over the last 3 hours, defaulting to the
  latest. The layer `RADAR_1KM_RDBR` does not exist and is not used.

**Rain or snow (a fixed rule):**

1. If the topic text says snow (or a snow word from a fixed list), use
   RSNO. If it says rain, use RRAI.
2. Otherwise, use RSNO from Nov 1 to Mar 31 at latitudes of 49° N and
   above, and RRAI otherwise. This season rule is a constant, so it is
   visible and testable.
3. The layer is always drawn on screen: "rain rate mm/h" or "snow rate
   cm/h", with the matching legend (`GetLegendGraphic`, style
   `Radar-Rain_14colors` or `Radar-Snow_14colors`). A wrong choice is
   therefore visible.

**"Live" follows the radar's own time:**

* The valid time comes from the layer's time dimension and is drawn as
  "Radar HH:MM" in local time.
* "Live" is shown only when that time is no more than 18 minutes (3
  steps) older than the HTTP `Date` header of the same response, which
  avoids trusting the local clock. Otherwise the strip says "Not live:
  radar from HH:MM", even after a successful fetch, because a 200
  response can carry an old default time during an ECCC outage.
* The map refreshes every 6 minutes, on the step. If a refresh fails, the
  image stays with the "Not live" label and the reason.

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

### Photos: Wikimedia Commons, curated and previewed

* **Curated categories only:** a search is limited to files in
  "Featured pictures on Wikimedia Commons", "Quality images" or "Valued
  images" (`incategory:`).
* **Excluded:**
  * a file in any category from a fixed blocklist (nudity, people,
    portraits, swimwear and similar);
  * a file whose structured data says it depicts a human (Q5).

  A missing statement does not prove there is no person, which is why
  every photo is previewed.
* **Licences:** public domain, CC0, and CC BY 2.0, 2.5, 3.0 and 4.0
  only. CC BY-SA, unknown licences and anything else are refused (no
  CC BY-SA and no Pexels, pending confirmation).
* **The choice is deterministic:** the query is built from the topic and
  the resolved place, and the app takes the highest-ranked result that
  passes every filter. The choice is cached per query for 30 days, so
  the same topic gives the same photo.
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
  * a refused licence or a filtered photo;
  * a place that is unknown, ambiguous or outside coverage;
  * the spend cap, or the setting being off.
* **The current background stays.** There is no silent fallback to
  another source, no stock image, and no old map labelled as current. A
  photo or a generated image is never labelled live.
* A failed attempt counts toward the cooldown, with one retry after 60
  s.

### Keys

* v1 sources need no key (GeoMet, OSM tiles, Commons, offline
  GeoNames). Imagine and speech-to-text use the existing xAI key from the
  OS credential store (`Secret::XaiKey`) or the environment.
* Any later key (a commercial tile provider) becomes a new `Secret` in
  `secrets.rs`. It is never stored in the project file, the settings, the
  chat, the prompt, logs or crash reports, and never passed as an AI
  tool argument.

### Tests (to write with the code; real fetches or recorded real responses, no fakes)

**Recorded fixtures** are real responses saved from the real services,
with the request URL and the fetch time beside them in
`tests/fixtures/background/`.

* The box this spec was written on cannot reach `geo.weather.gc.ca`
  with `curl`: the connection times out or the TLS handshake ends early.
  A web fetch tool can reach it, and the Auditor saw the same.
* So the GeoMet fixtures (GetCapabilities XML, a GetMap PNG, a legend
  PNG and a coverage GetFeatureInfo JSON) must be captured with `curl`
  from a machine whose network reaches the service, such as the owner's
  PC. Until then, those tests don't exist. Nothing is faked to stand in
  for them.

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
* `rain_snow_rule_picks_the_layer_and_labels_it`
* `place_outside_coverage_gives_the_no_radar_error`
* `eccc_credit_line_matches_the_licence_wording`

**Tiles, photos and failure** (a real local listener on 127.0.0.1 and
recorded real responses):

* `osm_tiles_send_the_user_agent_and_conditional_requests`
* `tile_url_setting_is_used`
* `commons_search_fixture_parses_author_licence_and_url`
* `commons_filter_refuses_sa_unknown_blocklisted_and_people`
* `commons_choice_is_deterministic`
* `commons_credit_strips_html_and_caps_length`
* `photo_always_needs_preview_even_in_automatic`
* `fetch_failure_keeps_the_current_background_and_shows_the_reason`: a
  real HTTP 503.
* `photos_and_generated_images_are_never_labelled_live`

**Imagine and keys:**

* `imagine_request_pins_model_ratio_and_resolution`
* `run_card_shows_the_usage_table_price_and_exact_prompt`
* `imagine_respects_the_spend_cap`
* `generated_background_is_cached_per_prompt`
* `keys_never_reach_project_settings_chat_or_logs`

**Speech:** `stt_fixture_transcript_feeds_topic_detection` (a recorded
real response from xAI's speech-to-text endpoint) and
`stt_chunk_is_a_valid_30_s_wav`.

**Network tests** are `#[ignore]`, skipped by default, and must be listed
in README's Tests section as skipped by default:

* `geomet_live_getmap_returns_a_png_for_edmonton`
* `geomet_live_valid_time_is_recent`
* `osm_live_tile_returns_200_with_cache_headers`
* `commons_live_curated_search_returns_an_allowed_licence`
* `xai_stt_live_transcribes_a_recorded_wav` (billed, run by hand)

A CI job that runs them on a schedule (`cargo test -- --ignored
<names>`) is **planned**. It can't run today: the organisation's GitHub
Actions are billing-locked. Until then, they are run by hand on a
machine with network, and the result is recorded in the PR.

### Open questions (still open)

The owner answered the four questions about money and schedule (see
"Decided" above). The Auditor's answers to the others are written in as
pending confirmation. Still to settle:

1. The blocklist of Commons categories (the exact list is to be reviewed
   with the owner before photos ship).
2. The rain/snow season constants (Nov 1 to Mar 31, at 49° N and
   above).

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve the public C ABI where a module ports OBS C code
- Fix the bugs in the safe path
- Delete before you add: no placeholder modules
- Every README claim is backed by a test or a measurement

## How to contribute
Clone, run `cargo test --lib` and `cargo test --bin ui`,
open a PR.
