// The carousel widget pack. The pack adds no key bindings: the browser
// scrolls a focused track, and each control is a native button or link.
// Set-up runs on `autumn:init`, one time for each carousel.
(function () {
  'use strict';

  var ready = new WeakSet();
  // The carousels that rotate now. A hidden page pauses them.
  var live = new Set();

  function reducedMotion() {
    return !!window.matchMedia &&
      window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  }

  // The direct children of `el` that have class `cls`.
  function kids(el, cls) {
    return Array.prototype.filter.call(el ? el.children : [], function (k) {
      return k.classList.contains(cls);
    });
  }

  // Fill each `{name}` slot of a text template.
  function fill(template, values) {
    return Object.keys(values).reduce(function (text, name) {
      return text.split('{' + name + '}').join(String(values[name]));
    }, template);
  }

  function rtl(c) {
    return getComputedStyle(c.track).direction === 'rtl';
  }

  // The left and right edges of the track content box.
  function box(c) {
    var t = c.track.getBoundingClientRect();
    var cs = getComputedStyle(c.track);
    return {
      left: t.left + (parseFloat(cs.paddingLeft) || 0),
      right: t.right - (parseFloat(cs.paddingRight) || 0)
    };
  }

  // Distance from the start edge of the content box to the start edge of
  // the slide. Physical edges work in both text directions.
  function delta(c, slide) {
    var b = box(c);
    var s = slide.getBoundingClientRect();
    return rtl(c) ? s.right - b.right : s.left - b.left;
  }

  // The slide nearest the start edge.
  function nearest(c) {
    var best = 0;
    var bestDistance = Infinity;
    c.slides.forEach(function (slide, i) {
      var d = Math.abs(delta(c, slide));
      if (d < bestDistance) {
        best = i;
        bestDistance = d;
      }
    });
    return best;
  }

  // The current slide: the slide that the user or rotation asked for, else
  // the slide nearest the start edge.
  function currentIndex(c) {
    return c.goal >= 0 ? c.goal : nearest(c);
  }

  // The first and the last slide (from 0) that show in full.
  function visible(c) {
    var b = box(c);
    var shown = [];
    c.slides.forEach(function (slide, i) {
      var s = slide.getBoundingClientRect();
      if (s.width > 0 && s.left >= b.left - 1 && s.right <= b.right + 1) shown.push(i);
    });
    return shown.length ? [shown[0], shown[shown.length - 1]] : null;
  }

  // How many slides one step moves: one slide, or one view of a rail.
  function measureStep(c) {
    if (c.slideshow) return 1;
    var range = visible(c);
    if (range) c.step = range[1] - range[0] + 1;
    return c.step;
  }

  function atFirst(c) {
    return currentIndex(c) <= 0;
  }

  // The last slide is in view from the current slide.
  function atLast(c) {
    return currentIndex(c) + c.step > c.slides.length - 1;
  }

  function setDisabled(button, disabled) {
    if (!button) return;
    if (disabled) button.setAttribute('aria-disabled', 'true');
    else button.removeAttribute('aria-disabled');
  }

  // Sync the markers, the buttons and `inert` with the current slide.
  function update(c) {
    var i = currentIndex(c);
    c.markers.forEach(function (m, k) {
      if (k === i) m.setAttribute('aria-current', 'true');
      else m.removeAttribute('aria-current');
    });
    if (c.slideshow) {
      c.slides.forEach(function (slide, k) {
        var out = k !== i;
        // Focus must not stay in a slide that becomes inert.
        if (out && !slide.inert && slide.contains(document.activeElement)) {
          c.track.focus({ preventScroll: true });
        }
        slide.inert = out;
      });
    }
    setDisabled(c.prev, !c.rewind && atFirst(c));
    setDisabled(c.next, !c.rewind && atLast(c));
    c.root.setAttribute('data-autumn-carousel-index', String(i));
    c.index = i;
  }

  // The scroll is at rest: sync, send the change event, and speak.
  function settle(c) {
    clearTimeout(c.settleTimer);
    c.settleTimer = 0;
    c.ours = false;
    measureStep(c);
    update(c);
    if (c.index !== c.emitted) {
      c.emitted = c.index;
      c.root.dispatchEvent(new CustomEvent('autumn-carousel:change', {
        bubbles: true,
        detail: { index: c.index }
      }));
    }
    if (c.speak) {
      c.speak = false;
      announce(c);
    }
  }

  // Scroll the track, and only the track, to slide `i`. An absolute
  // target: `scroll-snap-stop: always` stops a relative scroll at the
  // first snap point.
  function show(c, i, behavior) {
    c.goal = i;
    var max = c.track.scrollWidth - c.track.clientWidth;
    var from = c.track.scrollLeft;
    var to = from + delta(c, c.slides[i]);
    to = rtl(c) ? Math.min(0, Math.max(-max, to)) : Math.max(0, Math.min(max, to));
    update(c);
    if (Math.abs(to - from) < 1) {
      requestAnimationFrame(function () { settle(c); });
      return;
    }
    c.ours = true;
    c.track.scrollTo({ left: to, behavior: behavior || 'auto' });
  }

  function announce(c) {
    if (!c.status) return;
    var total = c.slides.length;
    var range = c.slideshow ? null : visible(c);
    if (range && range[0] !== range[1]) {
      c.status.textContent = fill(c.range, { first: range[0] + 1, last: range[1] + 1, total: total });
    } else {
      c.status.textContent = fill(c.position, { n: c.index + 1, total: total });
    }
  }

  // The target of one step. -1 when the step does nothing.
  function target(c, dir, wrap) {
    var last = c.slides.length - 1;
    var i = currentIndex(c);
    if (dir > 0) {
      if (atLast(c)) return wrap ? 0 : -1;
      return Math.min(i + c.step, last);
    }
    if (atFirst(c)) return wrap ? last : -1;
    return Math.max(i - c.step, 0);
  }

  // A move that the user asked for: it stops rotation and it speaks.
  function userMove(c, i) {
    if (i < 0) return;
    stop(c);
    c.speak = true;
    show(c, i);
  }

  // ── Rotation ───────────────────────────────────────────────────────────

  function paused(c) {
    return document.hidden || (c.hover && !c.userStarted);
  }

  // Focus on an element in the carousel, except the rotation button.
  function focusInside(c) {
    var a = document.activeElement;
    return !!a && a !== c.rotate && c.root.contains(a);
  }

  function schedule(c) {
    clearTimeout(c.timer);
    c.timer = 0;
    if (c.playing && !paused(c) && c.root.isConnected) {
      c.timer = setTimeout(function () { tick(c); }, c.interval);
    }
  }

  function tick(c) {
    c.timer = 0;
    if (!c.root.isConnected || reducedMotion()) {
      setPlaying(c, false);
      return;
    }
    // After a user start, focus in the carousel pauses rotation. The
    // focused content does not move away.
    if (!focusInside(c)) {
      var i = target(c, 1, true);
      if (i >= 0) show(c, i);
    }
    schedule(c);
  }

  function setPlaying(c, playing, byUser) {
    if (!c.rotate) return;
    c.playing = playing;
    c.userStarted = playing && !!byUser;
    c.root.setAttribute('data-autumn-carousel-state', playing ? 'playing' : 'stopped');
    if (c.label) {
      c.label.textContent = c.rotate.getAttribute(playing ? 'data-stop-label' : 'data-start-label');
    }
    if (playing) live.add(c);
    else live.delete(c);
    schedule(c);
  }

  function stop(c) {
    if (c.playing) setPlaying(c, false);
  }

  // ── Set-up ─────────────────────────────────────────────────────────────

  function setUp(root) {
    if (ready.has(root)) return;
    ready.add(root);
    var track = kids(root, 'autumn-carousel-track')[0];
    if (!track) return;
    var bar = kids(root, 'autumn-carousel-bar')[0];
    var group = kids(bar, 'autumn-carousel-markers')[0];
    var rotate = kids(bar, 'autumn-carousel-rotate')[0] || null;
    var status = kids(root, 'autumn-carousel-status')[0] || null;
    var interval = parseInt(root.getAttribute('data-autumn-carousel-rotate'), 10);
    var c = {
      root: root,
      track: track,
      slides: kids(track, 'autumn-carousel-slide'),
      markers: kids(group, 'autumn-carousel-marker'),
      prev: kids(bar, 'autumn-carousel-prev')[0] || null,
      next: kids(bar, 'autumn-carousel-next')[0] || null,
      rotate: rotate && interval > 0 ? rotate : null,
      label: rotate ? kids(rotate, 'autumn-carousel-label')[0] : null,
      status: status,
      position: (status && status.getAttribute('data-position')) || '{n} of {total}',
      range: (status && status.getAttribute('data-range')) || '{first} to {last} of {total}',
      slideshow: root.classList.contains('autumn-carousel-root--slideshow'),
      rewind: root.hasAttribute('data-autumn-carousel-rewind'),
      // The largest browser timer delay is 2^31 - 1 ms.
      interval: Math.min(Math.max(interval || 0, 1000), 2147483647),
      step: 1,
      goal: -1,
      index: -1,
      emitted: -1,
      ours: false,
      resizing: false,
      speak: false,
      pendingStart: 0,
      playing: false,
      userStarted: false,
      hover: false,
      timer: 0,
      settleTimer: 0,
      resizeTimer: 0,
      width: track.clientWidth,
      frame: 0
    };
    if (!c.slides.length) return;

    kids(bar, 'autumn-carousel-prev').concat(kids(bar, 'autumn-carousel-next'))
      .forEach(function (b) { b.hidden = false; });
    if (c.rotate) c.rotate.hidden = false;

    // A deep link to a slide wins. Else show the start slide. A hidden
    // carousel has no size: show the start slide when it gets one.
    var hash = location.hash.slice(1);
    var linked = !!hash && c.slides.some(function (s) { return s.id === hash; });
    var start = parseInt(root.getAttribute('data-autumn-carousel-start'), 10) || 0;
    if (linked || start <= 0 || start >= c.slides.length) start = 0;
    measureStep(c);
    if (start && track.clientWidth === 0) c.pendingStart = start;
    else if (start) show(c, start, 'instant');
    update(c);
    c.emitted = c.index;

    track.addEventListener('scroll', function () {
      // A scroll that the pack did not start is the user's move.
      if (!c.ours && !c.resizing) {
        c.goal = -1;
        stop(c);
      }
      if (!c.frame) {
        c.frame = requestAnimationFrame(function () {
          c.frame = 0;
          update(c);
        });
      }
      // Browsers without `scrollend` settle after a short rest.
      clearTimeout(c.settleTimer);
      c.settleTimer = setTimeout(function () { settle(c); }, 150);
    }, { passive: true });
    track.addEventListener('scrollend', function () { settle(c); });

    if (window.ResizeObserver) {
      var observer = new ResizeObserver(function () {
        if (!root.isConnected) {
          observer.disconnect();
          return;
        }
        if (track.clientWidth === 0) return;
        if (c.pendingStart) {
          var i = c.pendingStart;
          c.pendingStart = 0;
          c.width = track.clientWidth;
          measureStep(c);
          show(c, i, 'instant');
          return;
        }
        // A new width re-snaps the track. That scroll is not the user's.
        if (track.clientWidth === c.width) return;
        c.width = track.clientWidth;
        c.resizing = true;
        clearTimeout(c.resizeTimer);
        c.resizeTimer = setTimeout(function () { c.resizing = false; }, 150);
        measureStep(c);
        update(c);
      });
      observer.observe(track);
    }

    root.addEventListener('click', function (e) {
      var el = e.target.closest ? e.target.closest('button, a') : null;
      if (!el || el.closest('[data-autumn-carousel]') !== root) return;
      if (el === c.prev || el === c.next) {
        if (el.getAttribute('aria-disabled') === 'true') return;
        userMove(c, target(c, el === c.next ? 1 : -1, c.rewind));
      } else if (el === c.rotate) {
        setPlaying(c, !c.playing, true);
      } else if (c.markers.indexOf(el) >= 0) {
        // A modified click opens the link as the browser does.
        if (e.ctrlKey || e.metaKey || e.shiftKey || e.altKey || e.button !== 0) return;
        e.preventDefault();
        userMove(c, c.markers.indexOf(el));
      }
    });

    if (c.rotate) {
      // Focus stops rotation, except focus on the rotation button: the
      // user can then press it. After a user start, focus pauses it.
      root.addEventListener('focusin', function (e) {
        if (e.target !== c.rotate && !c.userStarted) stop(c);
      });
      root.addEventListener('pointerenter', function () {
        c.hover = true;
        schedule(c);
      });
      root.addEventListener('pointerleave', function () {
        c.hover = false;
        schedule(c);
      });
      setPlaying(c, !reducedMotion(), false);
    }
    root.setAttribute('data-autumn-carousel-ready', '');
  }

  function setUpAll(root) {
    if (!root || !root.querySelectorAll) return;
    if (root.matches && root.matches('[data-autumn-carousel]')) setUp(root);
    root.querySelectorAll('[data-autumn-carousel]').forEach(setUp);
  }

  document.addEventListener('autumn:init', function (e) { setUpAll(e.target); });

  // Without autumn-widgets.js there is no `autumn:init`. Set up when the
  // document is parsed, or now when it is parsed already.
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function () { setUpAll(document); });
  } else {
    setUpAll(document);
  }

  // A hidden page pauses rotation. A carousel that left the DOM stops.
  document.addEventListener('visibilitychange', function () {
    live.forEach(function (c) {
      if (c.root.isConnected) schedule(c);
      else setPlaying(c, false);
    });
  });
})();
