# Videos Directory

This directory is meant to contain video files that can be referenced in your SlideFlare presentations.

## Supported Formats

SlideFlare supports the following video formats:
- **MP4** (recommended) - `.mp4`
- **WebM** - `.webm`
- **AVI** - `.avi`
- **MOV** - `.mov`
- **OGV** - `.ogv`

## Usage Examples

In your markdown slides, you can reference videos like this:

### Basic Video
```html
<video controls width="600">
  <source src="./videos/my-video.mp4" type="video/mp4">
  Your browser does not support the video tag.
</video>
```

### Multiple Format Support
```html
<video controls width="600" height="400">
  <source src="./videos/presentation.mp4" type="video/mp4">
  <source src="./videos/presentation.webm" type="video/webm">
  <source src="./videos/presentation.ogv" type="video/ogg">
  Your browser does not support the video tag.
</video>
```

### Autoplay with Mute
```html
<video autoplay muted loop width="500">
  <source src="./videos/background-loop.mp4" type="video/mp4">
</video>
```

### With Poster Image
```html
<video controls poster="../images/video-thumbnail.jpg" width="600">
  <source src="./videos/main-content.mp4" type="video/mp4">
</video>
```

## Recommendations

1. **Use MP4 with H.264 encoding** for best compatibility
2. **Keep file sizes reasonable** - large videos may slow down your presentation
3. **Consider compression** - tools like FFmpeg can help optimize videos
4. **Test playback** - ensure videos work in your target environment

## Path Resolution

- Videos are referenced relative to your markdown file
- If your markdown is at `/presentations/slides.md` and you use `./videos/demo.mp4`, SlideFlare will look for the video at `/presentations/videos/demo.mp4`
