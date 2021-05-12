An attempt at vectorized point drawing. Port of some prior WebGL code to WebGPU. Thing is, it doesn't really work,
because WebGPU got rid of `gl_PointCoord` which was used to figure out where in the circle we were drawing in order to
discard pixels outside of the circle. So the POINTS geometry can be used for drawing rectangles, which is nice for
rendering little pixels, but not much else.

There is some discussion [here](https://github.com/gpuweb/gpuweb/issues/332) and
[here](https://github.com/gpuweb/gpuweb/issues/1190). tl;dr use instanced quads instead.