//! Browser tests in Chromium (AC-18 to AC-32).
//!
//! Run:
//!
//! ```text
//! cargo test --features system-tests --test browser -- --ignored --test-threads=1
//! ```

#![cfg(feature = "system-tests")]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::time::Duration;

use autumn_plugin_carousel::{ASSETS, CarouselConfig, Markers, PerView, Slide, carousel, pack};
use autumn_web::form::FieldOptions;
use autumn_web::plugin_conformance::widgets::browser::run_widget_browser_conformance;
use autumn_web::prelude::*;
use autumn_web::system_test::{Page, SystemTest, SystemTestRunner};
use maud::{DOCTYPE, Markup, html};

// ── Fixture pages ───────────────────────────────────────────────────────

/// A page with the core widget assets and the pack assets. With `core`
/// false, the page has no `autumn-widgets.js`.
fn page_with(dir: &str, core: bool, body: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" dir=(dir) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Carousel" }
                link rel="stylesheet" href=(autumn_web::ui::WIDGETS_CSS_PATH);
                @if core {
                    script src="/static/js/htmx.min.js" defer {}
                    script src="/static/js/autumn-widgets.js" defer {}
                }
                (pack().head())
            }
            body {
                main {
                    h1 { "Carousel test" }
                    a id="before" href="#top" { "Before" }
                    (*body)
                    a id="after" href="#top" { "After" }
                    // Room below the carousel, so a page scroll shows.
                    div class="autumn-visually-hidden" { "End" }
                }
            }
        }
    }
}

fn layout(dir: &str, body: &Markup) -> Markup {
    page_with(dir, true, body)
}

fn link_slides(count: usize) -> Vec<Slide<'static>> {
    (1..=count)
        .map(|n| {
            Slide::new(html! {
                h2 { "Story " (n) }
                p { "Body of story " (n) "." }
                a class="slide-link" href={ "/story/" (n) } { "Read story " (n) }
            })
        })
        .collect()
}

fn show(config: &CarouselConfig<'_>) -> Markup {
    carousel(&link_slides(4), config, &FieldOptions::new())
}

#[get("/slideshow")]
async fn slideshow_page() -> Markup {
    layout("ltr", &show(&CarouselConfig::new("show", "News")))
}

#[get("/rewind")]
async fn rewind_page() -> Markup {
    layout("ltr", &show(&CarouselConfig::new("show", "News").rewind()))
}

#[get("/rotate")]
async fn rotate_page() -> Markup {
    let config = CarouselConfig::new("show", "News").rotation_interval(Duration::from_secs(1));
    layout("ltr", &show(&config))
}

#[get("/rtl")]
async fn rtl_page() -> Markup {
    layout("rtl", &show(&CarouselConfig::new("show", "News")))
}

#[get("/start")]
async fn start_page() -> Markup {
    layout("ltr", &show(&CarouselConfig::new("show", "News").start(2)))
}

#[get("/bare")]
async fn bare_page() -> Markup {
    page_with("ltr", false, &show(&CarouselConfig::new("show", "News")))
}

#[get("/hidden-start")]
async fn hidden_start_page() -> Markup {
    layout(
        "ltr",
        &html! {
            div id="box" hidden { (show(&CarouselConfig::new("show", "News").start(2))) }
        },
    )
}

#[get("/rail")]
async fn rail_page() -> Markup {
    let config = CarouselConfig::new("rail", "Products")
        .rail(PerView::new(3))
        .markers(Markers::Numbers);
    layout(
        "ltr",
        &carousel(&link_slides(8), &config, &FieldOptions::new()),
    )
}

#[get("/swap")]
async fn swap_page() -> Markup {
    layout(
        "ltr",
        &html! {
            button id="load" type="button" hx-get="/swap/fragment" hx-target="#slot"
                hx-swap="innerHTML" { "Load" }
            div id="slot" {}
        },
    )
}

#[get("/swap/fragment")]
async fn swap_fragment() -> Markup {
    show(&CarouselConfig::new("show", "News"))
}

async fn runner() -> SystemTestRunner {
    Box::pin(
        SystemTest::new()
            .routes(routes![
                slideshow_page,
                rewind_page,
                rotate_page,
                rtl_page,
                start_page,
                bare_page,
                hidden_start_page,
                rail_page,
                swap_page,
                swap_fragment
            ])
            .plugin_assets(&ASSETS)
            .build(),
    )
    .await
    .expect("start the runner")
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Run `js`. Panic when it throws or returns another type.
async fn eval<T: serde::de::DeserializeOwned>(page: &Page, js: &str) -> T {
    page.evaluate(js)
        .await
        .unwrap_or_else(|e| panic!("`{js}` failed: {e}"))
        .into_value::<T>()
        .unwrap_or_else(|e| panic!("`{js}` gave another type: {e}"))
}

/// Poll `js` until it is true, for up to 5 s. An error reads as false.
async fn poll(page: &Page, js: &str) -> bool {
    for _ in 0..100 {
        let done = page
            .evaluate(js)
            .await
            .ok()
            .and_then(|r| r.into_value::<bool>().ok())
            .unwrap_or(false);
        if done {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

async fn assert_poll(page: &Page, js: &str, what: &str) {
    assert!(poll(page, js).await, "{what}: `{js}` did not become true");
}

/// Open `path` and wait until the carousel is ready.
async fn open(runner: &SystemTestRunner, path: &str) -> Page {
    let page = runner.page().await.expect("page");
    page.visit(path).await.expect("visit");
    assert_poll(&page, "document.readyState === 'complete'", "load").await;
    assert_poll(
        &page,
        "!!document.querySelector('[data-autumn-carousel-ready]')",
        "carousel set-up",
    )
    .await;
    page
}

/// The index (from 0) of the current marker, or -1.
const CURRENT: &str = "[...document.querySelectorAll('#show .autumn-carousel-marker')]\
    .findIndex(m => m.getAttribute('aria-current') === 'true')";

/// `true` when slide `N` (from 1) fills the content box of the track.
fn in_view(n: usize) -> String {
    format!(
        "(() => {{ const track = document.querySelector('#show .autumn-carousel-track'); \
         const t = track.getBoundingClientRect(); const cs = getComputedStyle(track); \
         const left = t.left + parseFloat(cs.paddingLeft); const right = t.right - parseFloat(cs.paddingRight); \
         const s = document.getElementById('show--slide-{n}').getBoundingClientRect(); \
         return Math.abs(s.left - left) < 2 && Math.abs(s.right - right) < 2; }})()"
    )
}

fn status_is(text: &str) -> String {
    format!("document.querySelector('#show .autumn-carousel-status').textContent === '{text}'")
}

const ROTATE_LABEL: &str =
    "document.querySelector('#show .autumn-carousel-rotate').textContent.trim()";

const STATE: &str = "document.getElementById('show').getAttribute('data-autumn-carousel-state')";

/// Focus the link in the slide that is not inert.
const FOCUS_CURRENT_LINK: &str = "(() => { \
    const link = document.querySelector('#show .autumn-carousel-slide:not([inert]) .slide-link'); \
    link.focus({ preventScroll: true }); return document.activeElement === link; })()";

async fn current(page: &Page) -> i64 {
    eval::<i64>(page, CURRENT).await
}

/// Wait `ms` and check that the current marker does not change.
async fn assert_still(page: &Page, ms: u64, what: &str) {
    let at = current(page).await;
    tokio::time::sleep(Duration::from_millis(ms)).await;
    assert_eq!(current(page).await, at, "{what}");
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac18_the_pack_passes_browser_conformance() {
    let axe = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/axe-core/axe.min.js"
    ))
    .expect("read the vendored axe-core");
    let report = run_widget_browser_conformance(&pack(), &axe)
        .await
        .expect("browser conformance runs");
    assert!(report.passed(), "{}", report.to_text_report());
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac19_without_js_all_slides_show_and_markers_work() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    page.set_javascript_enabled(false).await.expect("js off");
    page.visit("/slideshow").await.expect("visit");
    assert_poll(&page, "document.readyState === 'complete'", "load").await;
    assert!(eval::<bool>(&page, "window.__autumnWidgets === undefined").await);
    assert!(
        eval::<bool>(
            &page,
            "[...document.querySelectorAll('#show button')].every(b => b.hidden && !b.checkVisibility())"
        )
        .await,
        "the buttons must stay hidden without JS"
    );
    assert!(
        eval::<bool>(
            &page,
            "[...document.querySelectorAll('#show .autumn-carousel-slide')].every(s => s.checkVisibility() && !s.inert)"
        )
        .await,
        "each slide is rendered and active without JS"
    );
    assert!(
        eval::<bool>(
            &page,
            "(() => { const t = document.querySelector('#show .autumn-carousel-track'); return t.scrollWidth > t.clientWidth; })()"
        )
        .await,
        "the track scrolls"
    );
    assert!(
        eval::<bool>(&page, &in_view(1)).await,
        "slide 1 shows first"
    );
    // `Page::click` waits on page JS. Use a DOM click.
    page.evaluate("document.querySelector(\"a[href='#show--slide-3']\").click()")
        .await
        .expect("click marker");
    assert_poll(&page, &in_view(3), "the marker link shows slide 3").await;
    assert_poll(&page, "location.hash === '#show--slide-3'", "the fragment").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac19_the_track_has_room_for_focus_rings() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    assert!(
        eval::<bool>(
            &page,
            "parseFloat(getComputedStyle(document.querySelector('#show .autumn-carousel-track')).paddingTop) >= 2"
        )
        .await,
        "the track must not clip the focus ring of slide content"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac20_next_and_previous_move_announce_and_keep_focus() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    assert!(
        eval::<bool>(
            &page,
            "[...document.querySelectorAll('#show button')].every(b => !b.hidden)"
        )
        .await,
        "JS shows the buttons"
    );
    assert_eq!(current(&page).await, 0);
    assert!(eval::<bool>(&page, &status_is("")).await, "silent at load");

    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(&page, &format!("{CURRENT} === 1"), "next selects slide 2").await;
    assert_poll(&page, &in_view(2), "slide 2 in view").await;
    assert_poll(&page, &status_is("2 of 4"), "status after next").await;
    assert!(
        eval::<bool>(
            &page,
            "document.activeElement === document.querySelector('#show .autumn-carousel-next')"
        )
        .await,
        "focus stays on next"
    );

    page.click("#show .autumn-carousel-prev")
        .await
        .expect("prev");
    assert_poll(&page, &format!("{CURRENT} === 0"), "prev selects slide 1").await;
    assert_poll(&page, &in_view(1), "slide 1 in view").await;
    assert_poll(&page, &status_is("1 of 4"), "status after prev").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac20_quick_clicks_step_from_the_requested_slide() {
    let runner = runner().await;
    let page = open(&runner, "/rewind").await;
    page.evaluate(
        "(() => { const r = document.getElementById('show'); \
         r.querySelector('.autumn-carousel-next').click(); \
         setTimeout(() => r.querySelector('.autumn-carousel-prev').click(), 30); })()",
    )
    .await
    .expect("next, then prev");
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_poll(&page, &in_view(1), "next then prev ends on slide 1").await;
    assert_eq!(current(&page).await, 0);

    page.evaluate(
        "(() => { const n = document.querySelector('#show .autumn-carousel-next'); \
         n.click(); setTimeout(() => n.click(), 60); })()",
    )
    .await
    .expect("next twice");
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_poll(&page, &in_view(3), "two quick nexts end on slide 3").await;
    assert_poll(
        &page,
        &status_is("3 of 4"),
        "the status tells the end slide",
    )
    .await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac20_arrow_keys_on_the_track_scroll_natively() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    assert!(
        eval::<bool>(
            &page,
            "(() => { const t = document.querySelector('#show .autumn-carousel-track'); t.focus(); return document.activeElement === t; })()"
        )
        .await
    );
    page.press("ArrowRight").await.expect("ArrowRight");
    assert_poll(
        &page,
        &format!("{CURRENT} === 1"),
        "ArrowRight scrolls to slide 2",
    )
    .await;
    assert_poll(&page, &in_view(2), "slide 2 in view").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac21_buttons_are_disabled_at_the_ends_without_rewind() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    let prev_disabled = "document.querySelector('#show .autumn-carousel-prev').getAttribute('aria-disabled') === 'true'";
    let next_disabled = "document.querySelector('#show .autumn-carousel-next').getAttribute('aria-disabled') === 'true'";
    assert_poll(&page, prev_disabled, "prev is disabled at the start").await;
    assert!(!eval::<bool>(&page, next_disabled).await);

    page.click("#show .autumn-carousel-prev")
        .await
        .expect("prev");
    assert_still(&page, 400, "a disabled prev does nothing").await;

    for n in 1..=3 {
        page.click("#show .autumn-carousel-next")
            .await
            .expect("next");
        assert_poll(&page, &format!("{CURRENT} === {n}"), "next").await;
        assert_poll(&page, &in_view(n + 1), "the slide is in view").await;
    }
    assert_poll(&page, next_disabled, "next is disabled at the end").await;
    assert_poll(&page, &format!("!({prev_disabled})"), "prev is enabled").await;
    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_still(&page, 400, "a disabled next does nothing").await;
    assert_eq!(current(&page).await, 3);
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac21_rewind_wraps_at_the_ends() {
    let runner = runner().await;
    let page = open(&runner, "/rewind").await;
    assert!(
        eval::<bool>(
            &page,
            "!document.querySelector('#show [aria-disabled=\"true\"]')"
        )
        .await,
        "no button is disabled with rewind"
    );
    page.click("#show .autumn-carousel-prev")
        .await
        .expect("prev");
    assert_poll(&page, &format!("{CURRENT} === 3"), "prev wraps to the last").await;
    assert_poll(&page, &in_view(4), "slide 4 in view").await;
    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(
        &page,
        &format!("{CURRENT} === 0"),
        "next wraps to the first",
    )
    .await;
    assert_poll(&page, &in_view(1), "slide 1 in view").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac22_a_marker_click_shows_its_slide_and_keeps_the_url() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    let url = eval::<String>(&page, "location.href").await;
    let y = eval::<f64>(&page, "window.scrollY").await;
    page.click("a[href='#show--slide-3']")
        .await
        .expect("marker");
    assert_poll(&page, &format!("{CURRENT} === 2"), "marker 3 is current").await;
    assert_poll(&page, &in_view(3), "slide 3 in view").await;
    assert_poll(&page, &status_is("3 of 4"), "status after a marker").await;
    assert_eq!(eval::<String>(&page, "location.href").await, url);
    assert!(
        (eval::<f64>(&page, "window.scrollY").await - y).abs() < 1.0,
        "a marker click scrolls only the track"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac22_a_modified_marker_click_is_left_to_the_browser() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    // The test cancels the navigation itself. The pack must ignore a Ctrl
    // click: the browser opens the link in a new tab.
    page.evaluate(
        "(() => { const m = document.querySelector(\"a[href='#show--slide-3']\"); \
         m.addEventListener('click', e => e.preventDefault(), { once: true }); \
         m.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, ctrlKey: true })); })()",
    )
    .await
    .expect("ctrl click");
    assert_still(&page, 500, "a modified click does not move the track").await;
    assert_eq!(current(&page).await, 0);
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac22_start_and_deep_links_choose_the_first_slide() {
    let runner = runner().await;
    let page = open(&runner, "/start").await;
    assert_poll(&page, &in_view(3), "start(2) shows slide 3").await;
    assert_poll(&page, &format!("{CURRENT} === 2"), "marker 3 is current").await;

    let page = open(&runner, "/start#show--slide-2").await;
    assert_poll(&page, &in_view(2), "a deep link wins over start").await;
    assert_poll(&page, &format!("{CURRENT} === 1"), "marker 2 is current").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac22_start_works_when_the_carousel_shows_late() {
    let runner = runner().await;
    let page = open(&runner, "/hidden-start").await;
    page.evaluate("document.getElementById('box').hidden = false")
        .await
        .expect("show");
    assert_poll(
        &page,
        &in_view(3),
        "the start slide shows when the box shows",
    )
    .await;
    assert_poll(&page, &format!("{CURRENT} === 2"), "marker 3 is current").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac23_a_native_scroll_updates_the_current_marker() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    page.evaluate(
        "(() => { const t = document.querySelector('#show .autumn-carousel-track'); \
         t.scrollTo({ left: t.scrollWidth, behavior: 'instant' }); })()",
    )
    .await
    .expect("scroll");
    assert_poll(
        &page,
        &format!("{CURRENT} === 3"),
        "the marker follows the scroll",
    )
    .await;
    assert!(
        eval::<bool>(&page, &status_is("")).await,
        "a free scroll does not speak"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac24_slideshow_slides_out_of_view_are_inert() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    let inert =
        "[...document.querySelectorAll('#show .autumn-carousel-slide')].map(s => s.inert).join()";
    assert_poll(
        &page,
        &format!("{inert} === 'false,true,true,true'"),
        "start",
    )
    .await;
    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(
        &page,
        &format!("{inert} === 'true,false,true,true'"),
        "after next",
    )
    .await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac24_focus_is_not_lost_when_its_slide_leaves_the_view() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    assert!(eval::<bool>(&page, FOCUS_CURRENT_LINK).await);
    page.press("ArrowRight").await.expect("ArrowRight");
    assert_poll(&page, &format!("{CURRENT} === 1"), "the track scrolls").await;
    assert!(
        eval::<bool>(
            &page,
            "document.activeElement !== document.body && document.getElementById('show').contains(document.activeElement)"
        )
        .await,
        "focus must stay in the carousel"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac24_rail_cards_are_never_inert() {
    let runner = runner().await;
    let page = open(&runner, "/rail").await;
    assert!(
        eval::<f64>(&page, "window.innerWidth").await >= 768.0,
        "this test needs a viewport of 48rem or more"
    );
    let none_inert =
        "[...document.querySelectorAll('#rail .autumn-carousel-slide')].every(s => !s.inert)";
    assert_poll(&page, none_inert, "no card is inert").await;
    page.click("#rail .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(
        &page,
        "document.querySelector('#rail [aria-current=\"true\"]').getAttribute('href') === '#rail--slide-4'",
        "next moves one view of 3 cards",
    )
    .await;
    assert_poll(
        &page,
        "document.querySelector('#rail .autumn-carousel-status').textContent === '4 to 6 of 8'",
        "a rail speaks the cards in view",
    )
    .await;
    assert!(
        eval::<bool>(&page, none_inert).await,
        "no card is inert after a move"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac24_the_last_rail_cards_can_be_current() {
    let runner = runner().await;
    let page = open(&runner, "/rail").await;
    page.click("a[href='#rail--slide-8']")
        .await
        .expect("marker 8");
    assert_poll(
        &page,
        "document.querySelector('#rail [aria-current=\"true\"]').getAttribute('href') === '#rail--slide-8'",
        "marker 8 is current",
    )
    .await;
    assert_poll(
        &page,
        "document.querySelector('#rail .autumn-carousel-status').textContent === '6 to 8 of 8'",
        "the status tells the cards in view",
    )
    .await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac25_rotation_moves_without_focus_page_scroll_or_speech() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    let y = eval::<f64>(&page, "window.scrollY").await;
    assert_eq!(
        eval::<String>(&page, ROTATE_LABEL).await,
        "Stop automatic slide show"
    );
    assert_eq!(eval::<String>(&page, STATE).await, "playing");
    assert_poll(&page, &format!("{CURRENT} >= 1"), "rotation moves").await;
    assert!(eval::<bool>(&page, "document.activeElement === document.body").await);
    assert!(
        eval::<bool>(&page, &status_is("")).await,
        "rotation does not speak"
    );
    assert!(
        (eval::<f64>(&page, "window.scrollY").await - y).abs() < 1.0,
        "rotation scrolls only the track"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac25_rotation_wraps_to_the_first_slide() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    assert_poll(
        &page,
        &format!("{CURRENT} === 3"),
        "rotation reaches the end",
    )
    .await;
    assert_poll(&page, &format!("{CURRENT} === 0"), "rotation wraps").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac26_focus_stops_rotation_until_the_user_starts_it() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    assert!(
        eval::<bool>(&page, FOCUS_CURRENT_LINK).await,
        "focus a link"
    );
    assert_poll(
        &page,
        &format!("{ROTATE_LABEL} === 'Start automatic slide show'"),
        "focus stops rotation",
    )
    .await;
    assert_eq!(eval::<String>(&page, STATE).await, "stopped");
    assert_still(&page, 2500, "stopped rotation does not move").await;

    page.click("#show .autumn-carousel-rotate")
        .await
        .expect("start");
    assert_poll(
        &page,
        &format!("{ROTATE_LABEL} === 'Stop automatic slide show'"),
        "Start plays",
    )
    .await;
    let at = current(&page).await;
    assert_poll(
        &page,
        &format!("{CURRENT} !== {at}"),
        "focus on the rotation button does not stop a user start",
    )
    .await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac26_after_start_focus_in_a_slide_pauses_rotation() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    page.click("#show .autumn-carousel-rotate")
        .await
        .expect("stop");
    page.click("#show .autumn-carousel-rotate")
        .await
        .expect("start");
    assert_eq!(eval::<String>(&page, STATE).await, "playing");
    page.move_mouse(1.0, 1.0).await.expect("leave");
    assert!(
        eval::<bool>(&page, FOCUS_CURRENT_LINK).await,
        "focus a link"
    );
    assert_still(&page, 2500, "focus in a slide pauses a user start").await;
    assert_eq!(
        eval::<String>(&page, STATE).await,
        "playing",
        "a pause is not a stop"
    );
    assert!(
        eval::<bool>(
            &page,
            "document.activeElement.classList.contains('slide-link')"
        )
        .await,
        "rotation must not take focus away"
    );
    page.evaluate("document.getElementById('after').focus()")
        .await
        .expect("focus out");
    let at = current(&page).await;
    assert_poll(&page, &format!("{CURRENT} !== {at}"), "rotation resumes").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac26_a_click_on_stop_stops_rotation() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    page.click("#show .autumn-carousel-rotate")
        .await
        .expect("stop");
    assert_poll(
        &page,
        &format!("{ROTATE_LABEL} === 'Start automatic slide show'"),
        "Stop stops",
    )
    .await;
    page.move_mouse(1.0, 1.0).await.expect("leave");
    assert_still(&page, 2500, "a stopped carousel does not move").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac26_a_user_move_stops_rotation() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(
        &page,
        &format!("{ROTATE_LABEL} === 'Start automatic slide show'"),
        "a move stops rotation",
    )
    .await;
    assert_poll(&page, &status_is("2 of 4"), "a user move speaks").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac26_a_user_scroll_stops_rotation_and_a_vertical_wheel_does_not() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    page.evaluate(
        "document.querySelector('#show .autumn-carousel-track').dispatchEvent(\
         new WheelEvent('wheel', { deltaY: 120, bubbles: true }))",
    )
    .await
    .expect("vertical wheel");
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        eval::<String>(&page, STATE).await,
        "playing",
        "a page scroll over the track is not a slide move"
    );
    page.evaluate(
        "(() => { const t = document.querySelector('#show .autumn-carousel-track'); \
         t.scrollTo({ left: t.scrollWidth, behavior: 'instant' }); })()",
    )
    .await
    .expect("user scroll");
    assert_poll(
        &page,
        &format!("{STATE} === 'stopped'"),
        "a user scroll stops",
    )
    .await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac27_hover_pauses_rotation() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    page.hover("#show .autumn-carousel-track")
        .await
        .expect("hover");
    assert_still(&page, 2500, "hover pauses rotation").await;
    assert_eq!(
        eval::<String>(&page, STATE).await,
        "playing",
        "a pause is not a stop"
    );
    page.move_mouse(1.0, 1.0).await.expect("leave");
    let at = current(&page).await;
    assert_poll(&page, &format!("{CURRENT} !== {at}"), "leaving resumes").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac27_a_hidden_page_pauses_rotation() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    page.evaluate(
        "Object.defineProperty(document, 'hidden', { configurable: true, get: () => true }); \
         document.dispatchEvent(new Event('visibilitychange'));",
    )
    .await
    .expect("hide");
    assert_still(&page, 2500, "a hidden page pauses rotation").await;
    assert_eq!(
        eval::<String>(&page, STATE).await,
        "playing",
        "a pause is not a stop"
    );
    page.evaluate(
        "Object.defineProperty(document, 'hidden', { configurable: true, get: () => false }); \
         document.dispatchEvent(new Event('visibilitychange'));",
    )
    .await
    .expect("show");
    let at = current(&page).await;
    assert_poll(
        &page,
        &format!("{CURRENT} !== {at}"),
        "a shown page resumes",
    )
    .await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac28_reduced_motion_starts_rotation_stopped() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    // Ask for reduced motion, then set up a new copy of the carousel.
    page.evaluate(
        "(() => { \
           const real = window.matchMedia.bind(window); \
           window.matchMedia = q => q.includes('prefers-reduced-motion') \
             ? { matches: q.includes('reduce'), media: q, addEventListener() {}, removeEventListener() {} } \
             : real(q); \
           const old = document.getElementById('show'); \
           const copy = old.cloneNode(true); \
           copy.removeAttribute('data-autumn-carousel-ready'); \
           copy.querySelectorAll('button').forEach(b => { b.hidden = true; }); \
           old.replaceWith(copy); \
           copy.dispatchEvent(new CustomEvent('autumn:init', { bubbles: true })); \
         })()",
    )
    .await
    .expect("re-init");
    assert_poll(
        &page,
        "document.getElementById('show').hasAttribute('data-autumn-carousel-ready')",
        "the copy is set up",
    )
    .await;
    assert_eq!(
        eval::<String>(&page, ROTATE_LABEL).await,
        "Start automatic slide show"
    );
    assert_eq!(eval::<String>(&page, STATE).await, "stopped");
    assert_still(&page, 2500, "reduced motion does not rotate").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac29_an_htmx_swap_sets_up_the_carousel() {
    let runner = runner().await;
    let page = runner.page().await.expect("page");
    page.visit("/swap").await.expect("visit");
    assert_poll(
        &page,
        "document.readyState === 'complete' && !!window.htmx",
        "load",
    )
    .await;
    page.click("#load").await.expect("load");
    assert_poll(
        &page,
        "!!document.querySelector('#slot [data-autumn-carousel-ready]')",
        "the swapped carousel is set up",
    )
    .await;
    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(
        &page,
        &format!("{CURRENT} === 1"),
        "next works after a swap",
    )
    .await;
    page.expect_no_console_errors()
        .await
        .expect("no console errors");
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac29_a_page_without_the_core_runtime_sets_up_the_carousel() {
    let runner = runner().await;
    let page = open(&runner, "/bare").await;
    assert!(eval::<bool>(&page, "window.__autumnWidgets === undefined").await);
    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(&page, &format!("{CURRENT} === 1"), "next works").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac30_next_works_right_to_left() {
    let runner = runner().await;
    let page = open(&runner, "/rtl").await;
    let prev_disabled = "document.querySelector('#show .autumn-carousel-prev').getAttribute('aria-disabled') === 'true'";
    let next_disabled = "document.querySelector('#show .autumn-carousel-next').getAttribute('aria-disabled') === 'true'";
    assert!(
        eval::<bool>(&page, &in_view(1)).await,
        "slide 1 shows first"
    );
    assert_poll(&page, prev_disabled, "prev is disabled at the start").await;
    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(&page, &format!("{CURRENT} === 1"), "next selects slide 2").await;
    assert_poll(&page, &in_view(2), "slide 2 in view").await;
    assert!(
        eval::<bool>(
            &page,
            "document.getElementById('show--slide-2').getBoundingClientRect().left < \
             document.getElementById('show--slide-1').getBoundingClientRect().left"
        )
        .await,
        "slide 2 is on the left in right-to-left text"
    );
    for n in 2..=3 {
        page.click("#show .autumn-carousel-next")
            .await
            .expect("next");
        assert_poll(&page, &in_view(n + 1), "next").await;
    }
    assert_poll(&page, next_disabled, "next is disabled at the end").await;
    page.click("#show .autumn-carousel-prev")
        .await
        .expect("prev");
    assert_poll(&page, &format!("{CURRENT} === 2"), "prev selects slide 3").await;
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac31_a_change_event_fires_once_for_each_move() {
    let runner = runner().await;
    let page = open(&runner, "/slideshow").await;
    page.evaluate(
        "window.__changes = []; document.addEventListener('autumn-carousel:change', \
         e => window.__changes.push(e.detail.index + ':' + e.target.id));",
    )
    .await
    .expect("listen");
    page.click("#show .autumn-carousel-next")
        .await
        .expect("next");
    assert_poll(
        &page,
        "window.__changes.join() === '1:show'",
        "one change event",
    )
    .await;
    page.click("a[href='#show--slide-4']")
        .await
        .expect("marker 4");
    assert_poll(&page, &in_view(4), "slide 4 in view").await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(
        eval::<String>(&page, "window.__changes.join()").await,
        "1:show,3:show",
        "a long move sends one event"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac32_a_removed_carousel_stops_its_timer() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    assert_poll(&page, &format!("{CURRENT} >= 1"), "rotation runs").await;
    page.evaluate(
        "(() => { \
           document.getElementById('show').remove(); \
           const real = window.setTimeout; \
           window.__timers = 0; \
           window.setTimeout = function (fn, ms) { window.__timers++; return real(fn, ms); }; \
           document.dispatchEvent(new Event('visibilitychange')); \
         })()",
    )
    .await
    .expect("remove");
    tokio::time::sleep(Duration::from_millis(2500)).await;
    assert_eq!(
        eval::<i64>(&page, "window.__timers").await,
        0,
        "no new timer after removal"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac32_a_moved_carousel_shows_its_true_state() {
    let runner = runner().await;
    let page = open(&runner, "/rotate").await;
    page.evaluate(
        "(() => { const r = document.getElementById('show'); const p = r.parentNode; \
         const next = r.nextSibling; r.remove(); \
         setTimeout(() => p.insertBefore(r, next), 1300); })()",
    )
    .await
    .expect("move");
    assert_poll(
        &page,
        "!!document.getElementById('show') && document.getElementById('show').isConnected",
        "back in the DOM",
    )
    .await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let state = eval::<String>(&page, STATE).await;
    let label = eval::<String>(&page, ROTATE_LABEL).await;
    if state == "playing" {
        assert_eq!(label, "Stop automatic slide show");
        let at = current(&page).await;
        assert_poll(
            &page,
            &format!("{CURRENT} !== {at}"),
            "a playing carousel moves",
        )
        .await;
    } else {
        assert_eq!(state, "stopped");
        assert_eq!(label, "Start automatic slide show");
    }
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ac18_no_console_errors_on_each_page() {
    let runner = runner().await;
    for path in [
        "/slideshow",
        "/rewind",
        "/rotate",
        "/rtl",
        "/rail",
        "/start",
        "/bare",
    ] {
        let page = open(&runner, path).await;
        page.click(".autumn-carousel-next").await.expect("next");
        tokio::time::sleep(Duration::from_millis(300)).await;
        page.expect_no_console_errors()
            .await
            .unwrap_or_else(|e| panic!("{path}: {e}"));
    }
}
