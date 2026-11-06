// The carousel widget pack. It has no key code: the browser scrolls a
// focused track, and each control is a native button or link. Set-up runs
// on `autumn:init`, one time for each carousel.
(function () {
  'use strict';

  var ready = new WeakSet();
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

  // Distance from the start edge of the track to the start edge of the slide.
  // Physical edges work in both directions, so no `scrollLeft` sign math.
  function delta(c, slide) {
    var t = c.track.getBoundingClientRect();
    var s = slide.getBoundingClientRect();
    return getComputedStyle(c.track).direction === 'rtl' ? s.right - t.right : s.left - t.left;
  }

  function currentIndex(c) {
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

  function atStart(c) {
    return Math.abs(c.track.scrollLeft) <= 1;
  }

  function atEnd(c) {
    return Math.abs(c.track.scrollLeft) >= c.track.scrollWidth - c.track.clientWidth - 1;
  }

  // The number of slides that show in full. One at least.
  function visibleCount(c) {
    var t = c.track.getBoundingClientRect();
    var n = c.slides.filter(function (slide) {
      var s = slide.getBoundingClientRect();
      return s.left >= t.left - 1 && s.right <= t.right + 1;
    }).length;
    return Math.max(n, 1);
  }

  function setDisabled(button, disabled) {
    if (!button) return;
    if (disabled) button.setAttribute('aria-disabled', 'true');
    else button.removeAttribute('aria-disabled');
  }

  // Sync the markers, the buttons and `inert` with the scroll position.
  function update(c, quiet) {
    var i = currentIndex(c);
    c.markers.forEach(function (m, k) {
      if (k === i) m.setAttribute('aria-current', 'true');
      else m.removeAttribute('aria-current');
    });
    if (c.slideshow) {
      c.slides.forEach(function (slide, k) { slide.inert = k !== i; });
    }
    setDisabled(c.prev, !c.rewind && atStart(c));
    setDisabled(c.next, !c.rewind && atEnd(c));
    c.root.setAttribute('data-autumn-carousel-index', String(i));
    if (i === c.index) return;
    c.index = i;
    if (!quiet) {
      c.root.dispatchEvent(new CustomEvent('autumn-carousel:change', {
        bubbles: true,
        detail: { index: i }
      }));
    }
  }

  // Scroll the track, and only the track, to slide `i`. An absolute
  // target: `scroll-snap-stop: always` stops a relative scroll at the
  // first snap point.
  function show(c, i, behavior) {
    var d = delta(c, c.slides[i]);
    if (Math.abs(d) < 1) return;
    c.track.scrollTo({ left: c.track.scrollLeft + d, behavior: behavior || 'auto' });
  }

  function announce(c, i) {
    if (!c.status) return;
    c.status.textContent = c.position
      .replace('{n}', String(i + 1))
      .replace('{total}', String(c.slides.length));
  }

  // The target of one step. -1 when the step does nothing.
  function target(c, dir, wrap) {
    var last = c.slides.length - 1;
    var i = currentIndex(c);
    var step = c.slideshow ? 1 : visibleCount(c);
    if (dir > 0) {
      if (atEnd(c)) return wrap ? 0 : -1;
      return Math.min(i + step, last);
    }
    if (atStart(c)) return wrap ? last : -1;
    return Math.max(i - step, 0);
  }

  // A move that the user asked for: it stops rotation and it speaks.
  function userMove(c, i) {
    if (i < 0) return;
    stop(c);
    show(c, i);
    announce(c, i);
  }

  // ── Rotation ───────────────────────────────────────────────────────────

  function paused(c) {
    return document.hidden || (c.hover && !c.userStarted);
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
    if (!c.root.isConnected) {
      live.delete(c);
      return;
    }
    var i = target(c, 1, true);
    if (i >= 0) show(c, i);
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
      slideshow: root.classList.contains('autumn-carousel-root--slideshow'),
      rewind: root.hasAttribute('data-autumn-carousel-rewind'),
      interval: Math.max(interval || 0, 1000),
      index: -1,
      playing: false,
      userStarted: false,
      hover: false,
      timer: 0,
      frame: 0
    };
    if (!c.slides.length) return;

    kids(bar, 'autumn-carousel-prev').concat(kids(bar, 'autumn-carousel-next'))
      .forEach(function (b) { b.hidden = false; });
    if (c.rotate) c.rotate.hidden = false;

    // A deep link to a slide wins. Else show the start slide.
    var hash = location.hash.slice(1);
    var linked = hash && c.slides.some(function (s) { return s.id === hash; });
    var start = parseInt(root.getAttribute('data-autumn-carousel-start'), 10) || 0;
    if (!linked && start > 0 && start < c.slides.length) show(c, start, 'instant');
    update(c, true);

    track.addEventListener('scroll', function () {
      if (c.frame) return;
      c.frame = requestAnimationFrame(function () {
        c.frame = 0;
        update(c);
      });
    }, { passive: true });
    track.addEventListener('scrollend', function () { update(c); });
    if (window.ResizeObserver) {
      var observer = new ResizeObserver(function () {
        if (!root.isConnected) {
          observer.disconnect();
          return;
        }
        update(c, true);
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

    // A touch, a wheel or a key on the track is a user move.
    ['pointerdown', 'wheel', 'keydown'].forEach(function (type) {
      track.addEventListener(type, function () { stop(c); }, { passive: true });
    });

    if (c.rotate) {
      // Focus stops rotation, except focus on the rotation button: the
      // user can then press it. After a user start, only the button stops.
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
      live.add(c);
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

  // Without autumn-widgets.js there is no `autumn:init`. Set up on load.
  window.addEventListener('load', function () { setUpAll(document); });

  // A hidden page pauses rotation. Drop carousels that left the DOM.
  document.addEventListener('visibilitychange', function () {
    live.forEach(function (c) {
      if (c.root.isConnected) schedule(c);
      else live.delete(c);
    });
  });
})();
