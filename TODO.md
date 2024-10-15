# TODO

- Project view rays onto images and calculate integral over resulting line instead of sampling multiple points in space
- Filter images for [filtered back-projection](https://www.desy.de/~garutti/LECTURES/BioMedical/Lecture7_ImageReconstruction.pdf) (perhaps do this as well as -ln(sample) in a compute shader?)
- Map sample values to colors on a user-defined gradient for colored output
- Render scene to a texture and only re-render this when scene (or viewport) actually changes, such that eg. moving the mouse doesn't tank framerate.
- Rotate view with mouse
  - Render with lower settings while rotating for better framerate.
- High-res screenshots
- Image loading
  - Enforce uniform image dimensions at load-time (perhaps specify in scan descriptor file?)
  - Better error handling on image loading
- Support non-square render target without stretching image
