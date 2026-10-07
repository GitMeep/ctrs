# TODO

- ~~Support non-square render target without stretching image~~
- ~~Rotate view with mouse~~
  - ~~Render with lower settings while rotating for better framerate.~~
- ~~Render scene to a texture and only re-render this when scene (or viewport) actually changes, such that eg. moving the mouse doesn't tank framerate.~~
- Project view rays onto images and calculate integral over resulting line instead of sampling multiple points in space
- Map sample values to colors on a user-defined gradient for colored output.
  - Spline through HSV/RGB space?
- High-res screenshots
- Image loading
  - Enforce uniform image dimensions at load-time (perhaps specify in scan descriptor file?)
  - Better error handling on image loading
- Dynamic shader source loading at pipeline creation
  - Custom shaders
