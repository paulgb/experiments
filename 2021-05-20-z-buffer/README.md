Implementing z-buffering so that items are drawn front to back rather than back to
front. This should reduce the amount of excess fragment shading work when there
is a lot of overlap.

To test this, this experiment draws a specified number of circles, each of which travel
around in clip space. The frag shader for each circle renders the Julia fractal using its
center position as the input. Because this is a relatively slow frag shader, we can see
the benefits of the z-check.

To verify this, compare the reported FPS when run with z-buffering on (default) and without
(by passing `-d`). I find that passing `-n 40000` (to draw 40000 circles) makes the difference
pretty clear on my hardware (2017 MBP).