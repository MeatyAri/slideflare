---
bgColor: bg-gradient-to-br from-blue-500 to-purple-600
textColor: text-white
title: Introduction to Media Assets
---

# Welcome to SlideFlare

This slide demonstrates how **images** and **videos** are properly rendered using Tauri 2.0 asset protocol.

![Sample Image](./images/sample-image.jpg)

You can also use images with additional attributes:

<img src="./images/another-image.png" alt="Another sample" width="400" height="300" />

---
bgColor: bg-gradient-to-br from-green-500 to-teal-600
textColor: text-white
title: Video Examples
---

# Video Support

SlideFlare supports various video formats:

## Basic Video Tag

<video controls width="600" height="400">
  <source src="./videos/sample-video.mp4" type="video/mp4">
  <source src="./videos/sample-video.webm" type="video/webm">
  Your browser does not support the video tag.
</video>

## Autoplay Video (muted)

<video autoplay muted loop width="500">
  <source src="./videos/background-video.mp4" type="video/mp4">
</video>

---
bgColor: bg-gradient-to-br from-red-500 to-pink-600
textColor: text-white
title: Mixed Media Content
---

# Images and Videos Together

You can combine images and videos in the same slide:

![Diagram](./images/diagram.svg)

<video controls width="400">
  <source src="./videos/explanation.mp4" type="video/mp4">
</video>

## Features:
- ✅ Relative path support
- ✅ Absolute path support  
- ✅ Multiple video formats
- ✅ Image formats (jpg, png, svg, etc.)
- ✅ Proper asset URL conversion
- ✅ Tauri 2.0 compatibility

---
bgColor: bg-gradient-to-br from-yellow-500 to-orange-600
textColor: text-slate-800
title: Advanced Examples
---

# Advanced Media Usage

## Responsive Images

<picture>
  <source media="(min-width: 800px)" srcset="./images/large-image.jpg">
  <source media="(min-width: 400px)" srcset="./images/medium-image.jpg">
  <img src="./images/small-image.jpg" alt="Responsive image">
</picture>

## Video with Poster

<video controls poster="./images/video-poster.jpg" width="600">
  <source src="./videos/main-content.mp4" type="video/mp4">
  <source src="./videos/main-content.webm" type="video/webm">
</video>

*All asset paths are automatically converted to work with Tauri's secure asset protocol!*