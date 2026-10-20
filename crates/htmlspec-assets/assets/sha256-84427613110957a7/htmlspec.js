/*
 * Progressive enhancement for an HTMLSpec document. Vanilla JavaScript, no build
 * step, held under a byte budget the check suite enforces.
 *
 * The document is fully readable before this file runs. Everything here is an
 * improvement on a page that already works: a figure's caption and source are the
 * default state of the markup, not a <noscript> branch, so there is one thing to
 * get right instead of two that can disagree.
 *
 * THE ONE RULE THAT IS NOT NEGOTIABLE: a renderer draws into a SIBLING element and
 * never over the source. Mermaid's default replaces the source block with rendered
 * SVG, which destroys the source the moment anyone saves a rendered DOM — and a
 * saved page is exactly how these artifacts get shared. Sibling rendering makes the
 * source survive that round trip, and a test asserts it.
 */
(function () {
  "use strict";

  /* Where a rendered figure goes: a new element before the source, never the
   * source itself. Returning the element rather than rendering into the source is
   * the whole mechanism. */
  function renderTarget(source) {
    var holder = document.createElement("div");
    holder.className = "hs-figure-render";
    /* The figure, not the <details> wrapper, is the anchor: inserting inside the
     * wrapper would hide the rendering behind the disclosure the script is about
     * to collapse. */
    var figure = source.closest("figure") || source.parentNode;
    figure.insertBefore(holder, figure.firstChild);
    return holder;
  }

  /* Says a renderer failed, in place, rather than leaving an empty region. A
   * reader who sees nothing cannot tell a broken diagram from an absent one. */
  function reportFailure(source, what, error) {
    var note = document.createElement("p");
    note.className = "hs-figure-error";
    note.textContent =
      what + " did not render: " + (error && error.message ? error.message : error) +
      ". Its source is below.";
    var figure = source.closest("figure") || source.parentNode;
    figure.insertBefore(note, figure.firstChild);
  }

  /* Collapses a figure's source once the figure has rendered.
   *
   * Only when the rendering succeeded, and only for a <details> the author left
   * open. A source whose figure failed to render stays visible, because it is then
   * the only readable form of the content. */
  function collapseSource(source) {
    var wrapper = source.closest("details");
    if (wrapper && wrapper.open) {
      wrapper.open = false;
    }
  }

  /* Mermaid diagrams. */
  function renderDiagrams() {
    var sources = document.querySelectorAll("pre.mermaid");
    if (!sources.length || typeof window.mermaid === "undefined") {
      return;
    }
    /* startOnLoad is off because Mermaid's own scan replaces source blocks. The
     * rendering here is explicit, into a sibling, one figure at a time. */
    window.mermaid.initialize({ startOnLoad: false, securityLevel: "strict" });

    Array.prototype.forEach.call(sources, function (source, index) {
      var target = renderTarget(source);
      var id = "hs-mermaid-" + index;
      try {
        var result = window.mermaid.render(id, source.textContent, undefined);
        /* Mermaid 10+ returns a promise; earlier versions return the SVG. Both
         * are handled, because the pinned version is a project decision and this
         * file should not break when it moves within a major line. */
        if (result && typeof result.then === "function") {
          result.then(
            function (out) {
              target.innerHTML = out.svg || out;
              collapseSource(source);
            },
            function (error) {
              target.remove();
              reportFailure(source, "A diagram", error);
            }
          );
        } else {
          target.innerHTML = result && result.svg ? result.svg : result;
          collapseSource(source);
        }
      } catch (error) {
        target.remove();
        reportFailure(source, "A diagram", error);
      }
    });
  }

  /* Vega-Lite charts. */
  function renderCharts() {
    var sources = document.querySelectorAll("pre.vega-lite");
    if (!sources.length || typeof window.vegaEmbed === "undefined") {
      return;
    }
    Array.prototype.forEach.call(sources, function (source) {
      var target = renderTarget(source);
      var spec;
      try {
        spec = JSON.parse(source.textContent);
      } catch (error) {
        target.remove();
        reportFailure(source, "A chart specification", error);
        return;
      }
      /* actions: false removes the export menu, which would be an interactive
       * control in a document that has none, and whose "open in Vega editor"
       * item would send the specification to a remote service. */
      window
        .vegaEmbed(target, spec, { actions: false, renderer: "svg" })
        .then(
          function () {
            collapseSource(source);
          },
          function (error) {
            target.remove();
            reportFailure(source, "A chart", error);
          }
        );
    });
  }

  /* Marks the navigation entry for the section a reader is looking at.
   *
   * The navigation list is authored markup and its links resolve without this;
   * all that is added is which entry is current. IntersectionObserver rather than
   * a scroll handler, because a scroll handler on a long document is the kind of
   * cost a reading page should not pay. */
  function trackNavigation() {
    var links = document.querySelectorAll(".hs-nav a[href^='#']");
    if (!links.length || typeof window.IntersectionObserver === "undefined") {
      return;
    }
    var byId = {};
    var targets = [];
    Array.prototype.forEach.call(links, function (link) {
      var id = link.getAttribute("href").slice(1);
      var target = document.getElementById(id);
      if (target) {
        byId[id] = link;
        targets.push(target);
      }
    });
    if (!targets.length) {
      return;
    }

    var observer = new window.IntersectionObserver(
      function (entries) {
        entries.forEach(function (entry) {
          var link = byId[entry.target.id];
          if (!link) {
            return;
          }
          if (entry.isIntersecting) {
            link.setAttribute("aria-current", "true");
          } else {
            link.removeAttribute("aria-current");
          }
        });
      },
      { rootMargin: "-10% 0px -70% 0px" }
    );
    targets.forEach(function (target) {
      observer.observe(target);
    });
  }

  function enhance() {
    renderDiagrams();
    renderCharts();
    trackNavigation();
  }

  /* The script is deferred, so the document is usually parsed already. Both cases
   * are handled rather than assuming one. */
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", enhance);
  } else {
    enhance();
  }
})();
