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

## Topic backgrounds (spec; planned, not built)

The owner asked for this (relayed): "Either grok imagine or the ai wired,
or if it is a weather topic, the website for the live weather map of the
area speaking about. (As an example) or sunny topic a photo of a beach
from somewhere unless specific". Nothing in this section is built yet.
It is a spec. Weather and sunny are examples of one general rule: a topic
maps to a kind of source, and the background changes only when the topic
calls for it and a setting allows it.

### What exists today and what is missing

* Real today: the chat text (your messages and Seraphina's replies), the
  Grok chat model with tool calling (`src/bin/ui/llm.rs`, `AppCommand` in
  `src/bin/ui/commands.rs`), Grok Imagine image generation
  (`videogen.rs`, `grok-imagine-image-2.0`) behind the **Run** card and
  the spend cap (`usage.rs`), `ureq` for HTTP, the `image` crate, and the
  OS credential store (`secrets.rs`, used for the stream key and the xAI
  key).
* Missing: there is no background layer. The output source is one of
  Canvas, Screen or Camera (`output.rs`), with no compositing between
  them. There is no speech-to-text: the microphone goes straight into
  ffmpeg and nothing in the app hears it. There is no geocoder and no
  map code.

### Where the background goes

A new **Studio background** layer in the output compositor:

1. the background image, scaled to cover the output (1280×720 by default);
2. the camera picture on top, as a picture-in-picture box (a corner and
   size you choose) when the source is Camera, or the canvas inset when
   the source is Canvas;
3. an attribution strip along the bottom edge, drawn into every output
   frame whenever the background came from a third party (see Sources).

The same layer shows in the Stream preview, so what you see is what goes
out. Cutting the person out of the camera picture (segmentation) is later
and is not part of v1.

### Settings (off by default)

* **Topic backgrounds**: Off (default) / Ask me / Automatic. Off means no
  topic detection call, no fetch and no change, ever.
* Per-source switches: Live maps, Photos, Generated (Grok Imagine). All
  off until you turn them on.
* **Generated backgrounds**: Ask each time (default; the existing **Run**
  card with the price) / Allow up to N per stream. The spend cap still
  applies either way.
* **Pin background**: holds the current background until you unpin it.
* Changes go through one `AppCommand` (`set_studio_background`), so
  every change is undoable, and so the AI can only do what that command
  allows.

### Topic detection

* **Chat text (v1, real today).** When the setting is on, Seraphina gets
  a `set_studio_background` tool. Its arguments are `topic` (short
  text), `kind` (`live_weather`, `photo`, `generated`, `none`),
  `place` (optional text as said), and `confidence` (0 to 1). The model
  call that already handles the chat turn decides. It makes no extra
  call. A call with the setting off is refused and reported back to the
  model as a failure.
* **Speech (later in this feature, labelled planned until built).**
  The xAI speech-to-text API exists (REST `POST /v1/stt`, about $0.10 per
  hour of audio; streaming over `wss://api.x.ai/v1/stt`, about $0.20 per
  hour; prices checked on docs.x.ai at the time of writing). The proposal:
  * a second ffmpeg reads the same microphone device as the output
    settings (`dshow` on Windows, `pulse` on Linux) and writes 16 kHz mono
    s16le to stdout;
  * the app streams it to the xAI endpoint over a websocket (crate:
    `tungstenite`, MSRV to be checked against 1.88 before it is added);
  * every 30 s of new transcript, one small classification call returns
    the same fields as the tool.

  It is opt-in, its own switch, shown as "Transcribing your mic" while it
  runs, and billed through the existing meter. No audio is stored. The
  transcript is kept only in memory for the last 2 minutes.
* No keyword lists and no fake classifier. A topic comes from the model
  reading real text. If no model is configured, the feature says so and
  stays off.

### When the background changes: threshold, cooldown, hysteresis

* **Threshold:** a suggestion counts only if `confidence ≥ 0.75`.
* **Agreement:** the same `kind` and the same resolved place must come
  from two consecutive suggestions (two chat turns, or two 30 s speech
  windows), so one passing mention does nothing.
* **Hysteresis:** to replace the current background, the new topic's
  confidence must beat the current topic's last confidence by at least
  0.15. Going back to the default background needs three windows in a
  row with `kind = none` or no suggestion.
* **Cooldown:** at least 120 s between automatic changes, and at least
  60 s on screen for any background. A manual change resets the timer.
* **Ask me mode:** a card ("Switch background to: live weather radar,
  Edmonton, AB? [Switch] [Not now]") instead of an automatic change.
  "Not now" mutes that topic for 10 minutes.
* All five numbers are constants in one place, and tests check them.

### Location: extraction and ambiguity

* The model returns the place as said ("Edmonton", "Paris",
  "the coast"). The app resolves it. The model does not pick
  coordinates.
* **v1 geocoder: offline.** The app ships the GeoNames `cities15000`
  extract (about 25,000 places, CC BY 4.0, attribution in About and in
  the attribution strip when a place is shown). It needs no network,
  has no rate limit, and sends nothing to anyone.
* **Ambiguity:** if two or more places match and the largest is under
  10 times the population of the next one, the app asks ("Which Paris?
  Paris, France · Paris, Texas, US · Paris, Ontario, CA"). It never
  guesses silently. With one clear winner it shows the resolved name in
  the card and in the strip ("Paris, France"), so a wrong pick is
  visible and fixable in one click.
* **No place named:** live topics use the **home location** set in
  Settings (empty by default). If that is empty too, the app asks. Photo
  topics with no place use a general search ("beach"), as the owner
  asked.
* **Unknown place:** shown as an error ("Couldn't find 'Smallville'"),
  and the background stays as it is.
* Online geocoding (Nominatim, max 1 request/s, cached, with a
  User-Agent and a switchable endpoint) is later and is open question 2.

### Sources: the topic-to-source rule

| Topic kind | Source (v1) | Terms checked | On-screen attribution |
|------------|-------------|---------------|-----------------------|
| `live_weather`, place in Canada | ECCC MSC GeoMet WMS, radar layer `RADAR_1KM_RRAI` (rain) or `RADAR_1KM_RSNO` (snow), `GetMap` PNG for a box around the place | Environment and Climate Change Canada Data Servers End-use Licence: free, commercial use allowed, attribution required | "Radar: Environment and Climate Change Canada" plus the radar time |
| `live_weather`, place in the US | NOAA/NWS `opengeo.ncep.noaa.gov` WMS, `conus_bref_qcd` (base reflectivity mosaic) | US Government work, public domain | "Radar: NOAA/NWS" plus the radar time |
| `live_weather`, elsewhere | none in v1: an error says no licensed live radar source covers that place | RainViewer is free for personal and educational use only, and Seraphina Studio is a SynerGro product, so it is not used | n/a |
| Base map under the radar | OpenStreetMap standard tiles (`tile.openstreetmap.org`), zoom ≤ 10, cached 7 days, app User-Agent, no bulk prefetch | OSM Tile Usage Policy: allowed for normal use with attribution and caching; heavy or commercial use is told to use another provider (open question 1) | "© OpenStreetMap contributors" |
| `photo` | Wikimedia Commons API (`commons.wikimedia.org/w/api.php`, file search plus `imageinfo` `extmetadata`), with a User-Agent naming the app | Licence is per file. v1 accepts only public domain, CC0 and CC BY 2.0–4.0. It refuses CC BY-SA (open question 3), anything unknown, and files whose structured data says they depict a person | Author, licence short name, "Wikimedia Commons" |
| `generated` | Grok Imagine (`grok-imagine-image-2.0`, $0.04 per image at the time of writing) through the existing `generate_image` | xAI terms. The prompt is the topic text only. No chat history and no names | "Generated image (Grok Imagine)" |
| `generated`, later | the wired provider: Seraphina.AGI when it runs as a local process (after v0.2.0) | n/a | "Generated image (Seraphina)" |

Rules that hold for every source:

* **"Live" means live.** The word "Live" and a time stamp appear only on
  data fetched from a live source, with the source's own valid time (the
  WMS `TIME` dimension). A photo or a generated image is never labelled
  live.
* Live maps refresh every 10 minutes while shown. If a refresh fails,
  the strip changes to "Not live: last update HH:MM" and stays like that
  until a refresh succeeds.
* Attribution is drawn into the frame, not just shown in the app. A
  stream viewer sees it.
* The source's web page link (for example the ECCC radar page for that
  region) is shown in the app with an **Open website** button. It opens
  in your browser.
* Unsplash: not used. Its API guidelines (checked online) require
  non-automated use, hotlinking from its servers, a download ping, and
  keys that stay confidential (a proxy for a desktop app), and they
  forbid wallpaper-style apps. Pexels: its guidelines forbid apps that
  replicate wallpaper-style use of its photos and need an API key, so it
  is not in v1 (open question 3).

### Rendering a live map or page into the scene

egui has no browser engine, so v1 does not render web pages. It renders
the map from the same data the websites use:

1. one WMS `GetMap` request for the radar layer (PNG, transparent, an
   EPSG:3857 box about 400 km wide around the place, at the output size)
   over `ureq` (already a dependency);
2. the matching OSM tiles for the same box, stitched with the `image`
   crate (already a dependency);
3. radar drawn over the base map, a place marker, the place name, the
   radar's legend (WMS `GetLegendGraphic`) and the attribution strip;
4. uploaded once as an egui texture and handed to the output compositor
   as RGBA. No new crate is needed for v1.

Embedding a real website is later:

* **wry** (child webview, `WebViewBuilder::build_as_child`) can show a
  live page inside the Studio window. However, wry has no released
  offscreen or frame-capture API (the screenshot PR #1674 is still open),
  so the page can't be fed into the stream frames. It would be preview
  only.
* Capturing a browser window is the planned Window source (M7+, Stream).
  Whether a given site's terms allow re-broadcasting its page is checked
  per site when that lands.

### Cost and consent for Grok Imagine

* Off by default. When on, each generated background shows the existing
  **Run** card with the model and the price ($0.04 for
  `grok-imagine-image-2.0`), unless you chose "Allow up to N per stream".
* The spend cap (`usage::Meter`, `spend_cap_usd`) blocks the call with a
  toast once it is reached, as it does today for chat and Imagine.
* A generated background is cached per topic text for the session, so
  going back to a topic does not bill again.
* The prompt sent to Imagine is the topic text plus a fixed style line,
  shown on the card before you press **Run**.

### Failure: visible, and nothing passed off as live

* Any failure (no network, HTTP error, timeout of 10 s, decode error,
  licence refused, place unknown or ambiguous, spend cap reached,
  setting off) shows a toast and a status line with the real reason, for
  example "Background: GeoMet returned HTTP 503. Keeping the current
  background."
* **The current background stays.** There is no silent fallback to
  another source, no stock image, and no old map shown as current.
* A failed attempt counts toward the cooldown, so a down server is not
  hammered: one retry after 60 s, then wait for the next topic change.

### Keys

* v1 sources need no key (GeoMet, NWS, OSM tiles, Commons, offline
  GeoNames). Grok Imagine and speech-to-text use the xAI key that
  already lives in the OS credential store (`Secret::XaiKey`) or the
  environment.
* Any key a later source needs (a commercial tile or geocoding
  provider, Pexels) becomes a new `Secret` variant in `secrets.rs`. It is
  stored in Windows Credential Manager or a 0600 file, like the stream
  key. It is never stored in the project file, the settings file, the
  chat, the prompt, logs or crash reports, and it is not offered to the
  AI as a tool argument.

### Tests (to write with the code; real fetches or recorded real responses)

Recorded fixtures are real responses saved from the real services, with
the request URL and fetch time stored beside them in
`tests/fixtures/background/`. Network tests are `#[ignore]`, run by hand
or in a CI job with network, and are listed as such in README's Tests
section. None of them passes silently when skipped.

* `geomet_getmap_fixture_decodes_to_a_radar_image`: the recorded
  GeoMet PNG and `GetCapabilities` XML. Checks size, transparency, layer
  name and time dimension.
* `geomet_live_getmap_returns_a_png_for_edmonton` (network): a real
  GetMap. Checks status 200, `image/png` and that it decodes.
* `nws_getmap_fixture_decodes_to_a_radar_image` and
  `nws_live_getmap_returns_a_png` (network).
* `osm_tiles_send_the_app_user_agent_and_honour_the_cache`: a real local
  HTTP listener on 127.0.0.1 records the request headers. A second load
  within 7 days makes no request.
* `commons_search_fixture_parses_author_licence_and_url`: a recorded
  real API response.
* `commons_licence_filter_refuses_sa_unknown_and_people`.
* `gazetteer_resolves_edmonton_to_alberta`.
* `ambiguous_place_asks_instead_of_guessing` (Paris).
* `unknown_place_is_an_error_and_the_background_stays`.
* `no_place_uses_home_location_or_asks`.
* `low_confidence_suggestion_changes_nothing`.
* `two_agreeing_suggestions_are_needed_to_change`.
* `cooldown_blocks_a_second_change_within_120_s`.
* `hysteresis_keeps_the_background_on_a_brief_mention`.
* `setting_off_means_no_fetch_no_call_no_change`: the local listener
  counts zero requests, and the tool call is refused.
* `imagine_background_needs_run_and_respects_the_spend_cap`.
* `generated_background_is_cached_per_topic`.
* `fetch_failure_keeps_the_current_background_and_shows_the_reason`:
  the local listener returns a real HTTP 503.
* `stale_live_map_is_labelled_not_live`.
* `photos_and_generated_images_are_never_labelled_live`.
* `attribution_is_drawn_into_every_output_frame`: the strip's pixels
  are rendered with the expected text.
* `background_change_is_one_undo_step`.
* `keys_never_reach_project_settings_chat_or_logs`.
* Speech, when built: `stt_fixture_transcript_feeds_topic_detection`
  (a recorded real xAI STT response) and
  `xai_stt_live_transcribes_a_recorded_wav` (network, billed, by hand).

### Open questions

1. **Base map for a commercial product.** The OSM tile policy allows
   normal use with attribution, but tells heavy or commercial users to
   use another provider. Keep OSM tiles with a switchable URL, or pick a
   paid tile provider (and its key) before release?
2. **Online geocoding.** The offline GeoNames list covers cities over
   15,000 people. Small towns and landmarks need Nominatim (1 request/s,
   attribution, switchable endpoint; its policy has a clause on
   LLM-built software that the owner should read) or a paid geocoder.
   The free tier of Open-Meteo's geocoder is non-commercial only.
3. **Photo licences.** Allow CC BY-SA photos (scaling and cropping into
   a stream may count as an adaptation), and add Pexels with a key? Note
   that Pexels' and Unsplash's guidelines both restrict wallpaper-style
   use.
4. **Speech in v1?** Chat text is real today. Speech needs the xAI STT
   websocket, a new crate, and a billed, always-on stream while it runs.
   Ship it with this feature or right after?
5. **Weather outside Canada and the US.** No licensed live radar source
   is chosen. Show an error (v1 as written) or license one?
6. **Layout without segmentation.** Picture-in-picture camera over a full
   background is v1. Is that what the owner pictured, or is the person
   cut-out needed first?
7. **Automatic Imagine.** Should "Allow up to N per stream" exist at
   all, or must every generated background be approved on the card?
8. **Milestone.** This spec has no target date. Which milestone does it
   belong to?

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
