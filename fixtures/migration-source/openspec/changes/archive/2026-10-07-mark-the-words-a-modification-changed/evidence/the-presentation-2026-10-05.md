# The marks on the page, with scripts off and in print

**Date:** 2026-10-05
**Task:** 6.2
**Stylesheet:** `assets/sha256-ebf35a36fe4dc1a7/htmlspec.css`, the revision task 6.1 added. The print
stylesheet is unchanged, `sha256-04ac9a24fc436a87`.

The specification is the measurement's (appendix A of `the-measurement-2026-09-30.md`, the `ins-del`
variant), in a scratch project built with `init` from this change's binary, so it names the new
revision. It was rendered with Firefox, headless, in a profile with `javascript.enabled` set to
`false`, at 1100 by 1300 pixels, and each render was read as an image.

## With scripts off

- `<ins><code>sync</code>, </ins>` is drawn with a double underline under the code and its comma.
- `<del>and </del>` is struck through.
- `<ins>and its runbooks, </ins>` is drawn with a double underline.
- Neither start marker nor end marker is drawn: no "[insertion start]" or "[deletion start]" text
  appears anywhere on the page.

A link beside an inserted phrase was checked in a copy of the page, with a verification's method
written as `compare each with <a href="#req-…">the requirement</a>, <ins>and its runbooks</ins>
included.` The link is drawn in the accent color with one thin underline. The inserted phrase is
drawn in body ink with two. **An inserted word does not look like a link**, and the difference is
the number of lines, not only the hue.

## In print

Headless Firefox here has no print-to-PDF, so print was approximated: the print stylesheet's
`@media print` block was switched to `@media all` in a copy, and the page rendered with both
stylesheets, which is what paper gets. The navigation is gone and the ink is black, so the print
rules applied. The double underlines and the line-through are still drawn, in black, and neither
marker is drawn. A first attempt that only removed the `link` element's `media` attribute rendered
byte for byte what the screen render did, because the print rules sit inside that `@media print`
block; it is not counted.

## With a screen reader

**Run by the contributor on 2026-10-07**, with VoiceOver on macOS 26.6.2 and Safari, against the
same scratch specification. No screen reader can be driven from the session, so the contributor ran
the procedure and reported the result. VoiceOver was turned on with Command-F5, moved to the second
requirement by heading (Control-Option-Command-H), and stepped through its statement with
Control-Option-Right Arrow.

**All four markers were announced:** "deletion start" and "deletion end" around the deleted "and",
and "insertion start" and "insertion end" around the inserted "and its runbooks". So the stylesheet's
technique, which clips the markers to a 1-pixel box with `clip-path: inset(100%)` rather than hiding
them, keeps them audible while the page does not draw them.

Only VoiceOver with Safari was heard. Firefox with VoiceOver, and other screen readers, were not.
