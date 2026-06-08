---
bg_color: bg-gradient-to-br from-slate-950 via-indigo-950 to-slate-950
text_color: text-white text-center
title: Welcome
---

<div class="flex flex-col items-center justify-center text-center">

<span class="text-8xl mb-4">🔥</span>

<h1 class="text-7xl font-black mb-4 bg-gradient-to-r from-orange-400 via-amber-300 to-yellow-400 bg-clip-text text-transparent">
SlideFlare
</h1>

<p class="text-2xl text-white/70 mb-8">
Beautiful slides — powered by Markdown
</p>

<div class="flex gap-3 flex-wrap justify-center">
<span class="px-5 py-2 bg-white/10 backdrop-blur-sm rounded-full text-sm border border-white/20">📝 Markdown</span>
<span class="px-5 py-2 bg-white/10 backdrop-blur-sm rounded-full text-sm border border-white/20">🎨 Tailwind CSS</span>
<span class="px-5 py-2 bg-white/10 backdrop-blur-sm rounded-full text-sm border border-white/20">📐 LaTeX Math</span>
<span class="px-5 py-2 bg-white/10 backdrop-blur-sm rounded-full text-sm border border-white/20">🖼️ Media</span>
<span class="px-5 py-2 bg-white/10 backdrop-blur-sm rounded-full text-sm border border-white/20">⚡ Live Reload</span>
</div>

</div>

---
bg_color: bg-slate-900
text_color: text-white
title: What is SlideFlare?
---

## What is SlideFlare?

<div class="grid grid-cols-2 gap-6 max-w-4xl mx-auto mt-6">

<div class="p-6 bg-white/5 rounded-2xl border border-white/10">

### 📝 Markdown-First

Write slides in plain `.md` files. No drag-and-drop, no WYSIWYG editors — just clean, version-controllable text.

</div>

<div class="p-6 bg-white/5 rounded-2xl border border-white/10">

### 🎨 Styled with Tailwind

Every slide supports full Tailwind CSS utility classes for backgrounds, text, spacing, and custom layouts.

</div>

<div class="p-6 bg-white/5 rounded-2xl border border-white/10">

### 📐 LaTeX Math

Write beautiful equations inline or as display blocks — rendered via MathML.

</div>

<div class="p-6 bg-white/5 rounded-2xl border border-white/10">

### ⚡ Live Reload

Edit your slides and see changes instantly thanks to file watching.

</div>

</div>

---
bg_color: bg-gradient-to-r from-blue-600 to-cyan-600
text_color: text-white text-center
title: Simple Syntax
---

<div class="max-w-3xl mx-auto">

## One File, One Deck

Each slide is a Markdown section separated by `---`.

<div class="mt-8 p-6 bg-black/20 backdrop-blur-sm rounded-xl border border-white/20 text-left">

```markdown
---
bg_color: bg-slate-900
text_color: text-white
title: Slide Title
---

# Heading

Content goes here with **bold** and *italic*.

---
bg_color: bg-blue-600
text_color: text-white
title: Next Slide
---

More content...
```

</div>

<p class="mt-6 text-lg text-white/70">
That's it. Plain text. Any editor. Any workflow.
</p>

</div>

---
bg_color: bg-slate-900
text_color: text-white
title: Markdown Power
---

## Full Markdown Support

<div class="grid grid-cols-2 gap-6 max-w-4xl mx-auto">

<div class="space-y-4">

### Lists & Emphasis

- **Bold**, *italic*, ~~strikethrough~~
- Task lists: `- [ ]` and `- [x]`
- Ordered & unordered lists

### Code Blocks

```python
def hello():
    print("Hello, SlideFlare!")
```

</div>

<div class="space-y-4">

### Tables

| Feature | Status |
|---------|--------|
| Markdown | ✅ |
| Math | ✅ |
| Media | ✅ |

### Blockquotes

> The best slides are written, not drawn.

</div>

</div>

---
bg_color: bg-gradient-to-br from-indigo-900 via-purple-900 to-slate-900
text_color: text-white
title: LaTeX Math
---

## Beautiful Equations

Inline math: $E = mc^2$ and $\nabla \times \vec{E} = -\frac{\partial \vec{B}}{\partial t}$

## Display Math

<div class="mt-6 space-y-8 max-w-3xl">

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

$$
A v = \lambda v
$$

$$
J(\theta) = \sum_{i=1}^{m} \left( h_{\theta}(x^{(i)}) - y^{(i)} \right)^2
$$

</div>

<div class="mt-8 p-6 bg-white/5 rounded-xl border border-white/10">

**Syntax:**
- Inline: `$...$`
- Display: `$$...$$`

</div>

---
bg_color: bg-slate-800
text_color: text-white
title: HTML Components
---

## Custom Layouts with HTML

Embed arbitrary HTML with full Tailwind support.

<div class="grid grid-cols-2 gap-6 max-w-4xl mx-auto mt-6">

<div class="p-6 bg-gradient-to-br from-pink-500/20 to-rose-500/20 rounded-2xl border border-pink-400/30">

### Cards

Wrap content in styled containers:

```html
<div class="p-6 bg-white/10 rounded-xl border border-white/20">
## Card Content
</div>
```

</div>

<div class="p-6 bg-gradient-to-br from-emerald-500/20 to-teal-500/20 rounded-2xl border border-emerald-400/30">

### Grids

Multi-column layouts made easy:

```html
<div class="grid grid-cols-3 gap-4">
  <div>Column 1</div>
  <div>Column 2</div>
  <div>Column 3</div>
</div>
```

</div>

</div>

---
bg_color: bg-gradient-to-br from-amber-900 via-orange-900 to-red-900
text_color: text-white
title: Glassmorphism
---

## Glassmorphism Cards

<div class="mt-6 max-w-4xl mx-auto space-y-4">

<div class="p-8 bg-white/10 backdrop-blur-md rounded-2xl border border-white/20 shadow-2xl">

### ✨ Glass Effect

```html
<div class="p-8 bg-white/10 backdrop-blur-md rounded-2xl border border-white/20">
  ## Your Content Here
</div>
```

</div>

<div class="grid grid-cols-3 gap-4">

<div class="p-5 bg-white/10 backdrop-blur-sm rounded-xl border border-white/15 text-center">

**🎯** Precise

</div>

<div class="p-5 bg-white/10 backdrop-blur-sm rounded-xl border border-white/15 text-center">

**🎨** Beautiful

</div>

<div class="p-5 bg-white/10 backdrop-blur-sm rounded-xl border border-white/15 text-center">

**⚡** Fast

</div>

</div>

</div>

---
bg_color: bg-slate-900
text_color: text-white
title: Callout Boxes
---

## Highlighted Insights

<div class="space-y-4 max-w-3xl mx-auto mt-6">

<div class="p-5 bg-blue-500/15 border-l-4 border-blue-400 rounded-r-lg">

**💡 Key Insight:** SlideFlare uses plain Markdown files — version control your slides like code.

</div>

<div class="p-5 bg-emerald-500/15 border-l-4 border-emerald-400 rounded-r-lg">

**✅ Best Practice:** Use `bg-gradient-to-br` for visually striking backgrounds.

</div>

<div class="p-5 bg-amber-500/15 border-l-4 border-amber-400 rounded-r-lg">

**🔥 Pro Tip:** Combine `text-6xl` emojis with `text-center` for impactful title slides.

</div>

</div>

---
bg_color: bg-gradient-to-r from-violet-600 via-purple-600 to-indigo-700
text_color: text-white text-center
title: Media Support
---

<div class="max-w-4xl mx-auto">

## Images & Videos

<div class="grid grid-cols-2 gap-8 mt-6">

<div class="p-6 bg-white/10 backdrop-blur-sm rounded-2xl border border-white/20">

### 🖼️ Images

- Formats: PNG, JPG, GIF, SVG, WebP
- Embedded as Base64 data URLs
- Full Tailwind control on sizing

```markdown
![Alt text](./path/to/image.png)
```

</div>

<div class="p-6 bg-white/10 backdrop-blur-sm rounded-2xl border border-white/20">

### 🎬 Videos

- Formats: MP4, WebM, AVI, MOV, OGV
- Controls, autoplay, loop support
- Also embedded as Base64

```html
<video controls width="600">
  <source src="./video.mp4" type="video/mp4">
</video>
```

</div>

</div>

</div>

---
bg_color: bg-slate-800
text_color: text-white
title: Live Reload
---

## Edit & See Instantly

<div class="flex items-center justify-center gap-12 max-w-3xl mx-auto mt-8">

<div class="text-center">

<span class="text-7xl">✏️</span>

<h3 class="text-xl font-bold mt-3">Edit</h3>
<p class="text-gray-400 mt-2">Change your `.md` file in any editor</p>

</div>

<span class="text-5xl text-white/30">→</span>

<div class="text-center">

<span class="text-7xl">👀</span>

<h3 class="text-xl font-bold mt-3">Preview</h3>
<p class="text-gray-400 mt-2">Changes appear instantly</p>

</div>

</div>

<div class="mt-10 p-6 bg-white/5 rounded-xl border border-white/10 max-w-2xl mx-auto text-center">

```bash
bun run tauri dev   # Start with hot reload
bun run dev         # Frontend only
```

</div>

---
bg_color: bg-gradient-to-br from-emerald-600 to-teal-700
text_color: text-white
title: Color Themes
---

## Endless Color Combinations

<div class="grid grid-cols-2 gap-6 max-w-4xl mx-auto mt-6">

<div class="p-5 bg-white/10 backdrop-blur-sm rounded-xl border border-white/20 text-center">

`bg-gradient-to-r from-amber-400 to-orange-500`

<h4 class="mt-2 font-bold">🌅 Warm</h4>

</div>

<div class="p-5 bg-white/10 backdrop-blur-sm rounded-xl border border-white/20 text-center">

`bg-gradient-to-br from-violet-600 to-purple-800`

<h4 class="mt-2 font-bold">🔮 Creative</h4>

</div>

<div class="p-5 bg-white/10 backdrop-blur-sm rounded-xl border border-white/20 text-center">

`bg-gradient-to-r from-emerald-500 to-teal-600`

<h4 class="mt-2 font-bold">🌿 Fresh</h4>

</div>

<div class="p-5 bg-white/10 backdrop-blur-sm rounded-xl border border-white/20 text-center">

`bg-slate-900`

<h4 class="mt-2 font-bold">🌑 Classic Dark</h4>

</div>

</div>

---
bg_color: bg-gradient-to-br from-amber-400 via-orange-500 to-red-500
text_color: text-white text-center
title: Get Started
---

<div class="flex flex-col items-center">

<span class="text-8xl mb-6">🚀</span>

## Ready to Create?

<p class="text-2xl mb-8 text-white/90 max-w-xl">
Write your first slide in under a minute. No design skills required.
</p>

<div class="space-y-4 text-left max-w-lg">

<div class="p-4 bg-white/15 backdrop-blur-sm rounded-xl border border-white/20">

**1.** Create a `.md` file with YAML frontmatter

</div>

<div class="p-4 bg-white/15 backdrop-blur-sm rounded-xl border border-white/20">

**2.** Write Markdown content between `---` dividers

</div>

<div class="p-4 bg-white/15 backdrop-blur-sm rounded-xl border border-white/20">

**3.** Run SlideFlare and watch your slides come alive

</div>

</div>

<div class="mt-10 text-3xl font-bold">
Let your ideas flare ✨
</div>

</div>

