# Acceptance evidence

This file maps each acceptance criterion in [design.md §9](design.md#9-acceptance-criteria)
to its evidence. Date: 2026-11-06.

Test files: `M` = `tests/markup.rs`, `P` = `tests/properties.rs`,
`C` = `tests/conformance.rs`, `B` = `tests/browser.rs` (Chromium).

## Results

| Gate | Command | Result |
|---|---|---|
| Rust tests | `cargo test` | 51 tests and 6 doctests pass |
| Browser tests | `cargo test --features system-tests --test browser -- --ignored --test-threads=1` | 34 pass |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` | no warning |
| Format | `cargo fmt --all --check` | clean |
| Docs | `RUSTDOCFLAGS=-D warnings cargo doc --no-deps` | no warning |
| Coverage | `cargo llvm-cov --fail-under-lines 90 --ignore-filename-regex 'tests/'` | 99.8% lines, 100% functions |
| Style | `python3 -I scripts/check-ste-length.py …` (see CI) | 0 sentences over 25 words |

## Markup and API

| AC | Evidence |
|---|---|
| AC-01 | M `ac01_root_is_a_named_carousel_section`, `ac01_labelled_by_replaces_the_label`, `ac01_an_empty_label_panics_in_a_debug_build`, `ac01_an_empty_label_is_allowed_with_labelled_by` |
| AC-02 | M `ac02_slides_are_named_groups`, `ac02_slide_content_is_kept`; P `ac14_markup_rules_hold` (names for 0–11 slides) |
| AC-03 | M `ac03_ids_derive_from_the_root_id_and_do_not_repeat`, `ac03_ids_of_two_carousels_do_not_collide`; P `ac14_markup_rules_hold` |
| AC-04 | M `ac04_buttons_are_hidden_named_and_control_the_track`; P (all buttons `hidden`) |
| AC-05 | M `ac05_rotation_button_comes_first_and_only_with_rotation`, `ac05_controls_come_before_the_track_in_dom_order` |
| AC-06 | M `ac06_markers_link_to_slides_and_mark_the_start`, `ac06_default_markers_depend_on_the_layout`; P (one `aria-current`, on the start marker) |
| AC-07 | M `ac07_thumbnails_use_the_slide_image_or_the_override` |
| AC-08 | M `ac08_images_have_size_alt_and_lazy_loading`, `ac08_a_caption_on_a_markup_slide_is_ignored` |
| AC-09 | M `ac09_no_slides_give_no_markup`, `ac09_one_slide_has_no_controls`, `ac09_many_slides_have_a_live_region`; P |
| AC-10 | M `ac10_start_is_clamped_and_marked`; P (start index 0–19 against 0–11 slides) |
| AC-11 | M `ac11_all_text_comes_from_carousel_text`, `ac11_default_position_text`; doctest on `CarouselText` |
| AC-12 | M `ac12_root_gets_the_app_passthrough`, `ac12_widget_attributes_win_over_the_passthrough` |
| AC-13 | M `ac13_try_new_returns_an_error`, `ac13_valid_ids_do_not_panic`, `ac13_an_id_with_a_space_panics`, `ac13_an_empty_id_panics`, `ac13_an_id_with_a_double_dash_panics`; doctest on `try_new` |
| AC-14 | M `ac14_markup_is_csp_safe`, `ac14_text_is_escaped`; P `ac14_markup_rules_hold`; C `ac15_…` (`widget-markup-csp` check) |

## Pack and plugin

| AC | Evidence |
|---|---|
| AC-15 | C `ac15_the_pack_passes_static_conformance`: `widget-pack-name`, `widget-stories`, `widget-class-names`, `widget-markup-csp`, `widget-tokens`, `widget-scripts` pass |
| AC-16 | C `ac16_each_variant_has_a_story`: slideshow, auto rotation, rail, gallery, numbers |
| AC-17 | C `ac17_the_bundle_holds_the_css_and_js`, `ac17_the_plugin_serves_the_assets`, `ac17_the_plugin_declares_its_contract` (`evaluate` gives `Compatible` for 0.10, `Incompatible` for 0.11), `ac17_installing_twice_is_safe`, `ac17_head_links_the_pack_assets` |

## Browser

| AC | Evidence |
|---|---|
| AC-18 | B `ac18_the_pack_passes_browser_conformance` (axe in light and dark, console, keyboard), `ac18_no_console_errors_on_each_page` |
| AC-19 | B `ac19_without_js_all_slides_show_and_markers_work`, `ac19_the_track_has_room_for_focus_rings`; M `ac19_track_is_a_named_focusable_group` |
| AC-20 | B `ac20_next_and_previous_move_announce_and_keep_focus`, `ac20_arrow_keys_on_the_track_scroll_natively` |
| AC-21 | B `ac21_buttons_are_disabled_at_the_ends_without_rewind`, `ac21_rewind_wraps_at_the_ends` |
| AC-22 | B `ac22_a_marker_click_shows_its_slide_and_keeps_the_url` (URL and `scrollY` do not change), `ac22_a_modified_marker_click_is_left_to_the_browser` |
| AC-23 | B `ac23_a_native_scroll_updates_the_current_marker` |
| AC-24 | B `ac24_slideshow_slides_out_of_view_are_inert`, `ac24_rail_cards_are_never_inert` |
| AC-25 | B `ac25_rotation_moves_without_focus_page_scroll_or_speech`, `ac25_rotation_wraps_to_the_first_slide` |
| AC-26 | B `ac26_focus_stops_rotation_until_the_user_starts_it`, `ac26_a_click_on_stop_stops_rotation`, `ac26_a_user_move_stops_rotation` |
| AC-27 | B `ac27_hover_pauses_rotation` |
| AC-28 | B `ac28_reduced_motion_starts_rotation_stopped` |
| AC-29 | B `ac29_an_htmx_swap_sets_up_the_carousel` |
| AC-30 | B `ac30_next_works_right_to_left` (also `aria-disabled` at both ends) |
| AC-31 | B `ac31_a_change_event_fires_once_for_each_move` |
| AC-32 | B `ac32_a_removed_carousel_stops_its_timer`, `ac32_a_moved_carousel_shows_its_true_state` |

## Quality

| AC | Evidence |
|---|---|
| AC-33 | The lint, format and docs gates above. `Cargo.toml` turns on `clippy::pedantic` and `clippy::nursery`. No `unwrap` or `expect` in `src/`. |
| AC-34 | `.github/workflows/ci.yml`: `style`, `fmt`, `clippy`, `test` (Linux, macOS, Windows), `doc`, `browser` (pass-count check, failure artifacts), `coverage` (`--fail-under-lines 90`). |
| AC-35 | `scripts/check-ste-length.py` runs in CI. A docs review rewrote the passages that were not STE ([review fixes](../CHANGELOG.md)). |
| AC-36 | The coverage gate above. CI fails below 90%. |

## Added after the review

| AC | Evidence |
|---|---|
| AC-37 | B `ac20_quick_clicks_step_from_the_requested_slide` |
| AC-38 | B `ac24_focus_is_not_lost_when_its_slide_leaves_the_view`, `ac26_after_start_focus_in_a_slide_pauses_rotation` |
| AC-39 | B `ac24_the_last_rail_cards_can_be_current`, `ac24_rail_cards_are_never_inert` ("4 to 6 of 8") |
| AC-40 | B `ac26_a_user_scroll_stops_rotation_and_a_vertical_wheel_does_not` |
| AC-41 | M `d4_rotation_interval_has_a_ceiling`, `d4_rotation_interval_has_a_floor` |
| AC-42 | M `ac06_a_number_marker_name_holds_its_digit` |
| AC-43 | B `ac27_a_hidden_page_pauses_rotation` |
| AC-44 | B `ac22_start_and_deep_links_choose_the_first_slide`, `ac22_start_works_when_the_carousel_shows_late` |
| AC-45 | B `ac29_a_page_without_the_core_runtime_sets_up_the_carousel` |
| AC-46 | B `ac19_the_track_has_room_for_focus_rings` |

## Known limits

These are design decisions, not gaps. [design.md §4](design.md#4-six-thinking-hats)
gives the reasons.

- With JS off, `aria-current` stays on the start marker.
- The controls come before the track in the DOM but show below it.
- Find-in-page does not reach slideshow slides that are `inert`.
- Forced colors and Firefox and Safari behavior have no automated test.
  The CI browser is Chromium. On 2026-11-06, a Playwright run in emulated
  forced colors showed these cues for the current marker:
  - A thumbnail gets a `Highlight` border and an outline. The others get no
    outline.
  - A number gets a `Highlight` fill.
  - A dot is 30 px wide and filled. The others are 12 px wide.
- `ac28` stubs `matchMedia`, because the Autumn test `Page` cannot emulate
  media features.
