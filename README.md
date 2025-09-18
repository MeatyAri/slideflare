<div align="center">

# 🔥 SlideFlare

**Blazing fast, interactive presentation tool for developers, educators, and creators**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](https://github.com/MeatyAri/slideflare/releases)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/frontend-Svelte-red.svg)](https://svelte.dev/)

*Build and share beautiful slides effortlessly with markdown, math, multimedia, and more*

[📥 Download](#installation) • [📖 Documentation](https://github.com/MeatyAri/slideflare/wiki) • [🎯 Examples](#examples) • [🚀 Quick Start](#quick-start)

</div>

---

## ✨ Features

### 🔥 **Performance First**
- **Rust-powered backend** for lightning-fast rendering and file processing
- **Optimized slide parsing** with efficient caching and hot-reloading
- **Minimal resource usage** - perfect for resource-constrained environments

### 📝 **Markdown Native**
- Write presentations in **pure Markdown** with YAML frontmatter
- **Live preview** with instant updates as you type
- **Syntax highlighting** for code blocks
- **GitHub Flavored Markdown** support

### 🧮 **Mathematical Excellence**
- **LaTeX math rendering** with pure Rust implementation converting to MathML via pulldown-latex
- Inline math: `$E = mc^2$` and display math: `$$\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}$$`
- **Mathematical symbols** and complex equations

### 🎨 **Rich Media Support**
- **Images**: PNG, JPG, GIF, SVG, WebP
- **Videos**: MP4, WebM, AVI, MOV, OGV with autoplay and controls
- **Custom styling** with Tailwind CSS classes
- **Responsive design** that adapts to any screen size

### 🎯 **Developer Experience**
- **Drag & drop** markdown files to instantly create presentations
- **Hot reload** during development with `bun run tauri dev`
- **Cross-platform** - Windows, macOS, and Linux support
- **Extensible** architecture for custom themes and plugins

### 🚀 **Coming Soon: AI-Powered**
- **Smart conversion** from PowerPoint and PDF to SlideFlare format
- **AI-assisted slide creation** and content optimization
- **OCR integration** for extracting content from existing presentations
- **Automated styling** and layout suggestions

## 🚀 Quick Start

### 1. Create Your First Slide

Create a markdown file `my-presentation.md`:

<pre>
---
bg_color: bg-gradient-to-br from-blue-600 to-purple-700
text_color: text-white
title: My Amazing Presentation
---

# Welcome to SlideFlare! 🔥

- ✨ Beautiful presentations
- 🚀 Lightning fast
- 📝 Markdown powered

Let's build something amazing together!

---
bg_color: bg-green-600
text_color: text-white
---

## Code Example

Here's some beautiful syntax-highlighted code:

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

print([fibonacci(i) for i in range(10)])
```

---
bg_color: bg-purple-600
text_color: text-white text-center
---

## Math is Beautiful

The quadratic formula:

$$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$$

And inline math works too: $\sum_{i=1}^n i = \frac{n(n+1)}{2}$
---
</pre>

### 2. Launch SlideFlare

Simply **drag and drop** your markdown file into the SlideFlare application, and watch your presentation come to life!

## 📥 Installation

### Download Prebuilt Binaries

Visit our [releases page](https://github.com/MeatyAri/slideflare/releases) to download the latest version:

- **Linux**: `slideflare-linux-x64`
- **macOS**: `slideflare-macos-universal.dmg`
- **Windows**: `slideflare-windows-x64.exe`

### Linux & macOS
```bash
chmod +x slideflare
./slideflare
```

### Windows
Double-click the downloaded executable to run.

### Build from Source

**Prerequisites:**
- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) or [Node.js](https://nodejs.org/) 18+
- [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

```bash
# Clone the repository
git clone https://github.com/MeatyAri/slideflare.git
cd slideflare

# Install dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```

## 🎯 Examples

### Basic Slide Structure

```markdown
---
bg_color: bg-slate-900
text_color: text-white
title: My Slide Title
---

# Main Heading

Content goes here...

- Bullet point 1
- Bullet point 2

## Subheading

More content...

---
# Next slide starts here
```

### Advanced Features

#### Mathematical Expressions
```markdown
# Physics Formula

Einstein's mass-energy equivalence:
$$E = mc^2$$

Where:
- $E$ = energy
- $m$ = mass
- $c$ = speed of light
```

#### Media Integration
```markdown
# Rich Media

![Beautiful Image](./images/photo.jpg)

<video controls width="600">
  <source src="./videos/demo.mp4" type="video/mp4">
</video>
```

#### Custom Styling
```markdown
---
bg_color: bg-gradient-to-r from-cyan-500 to-blue-500
text_color: text-white text-center
---

<div class="card bg-white/20 p-8 rounded-xl">

# Styled Content

This slide has a gradient background and a semi-transparent card.

</div>
```

## 🛠️ Development

### Tech Stack

- **Backend**: Rust with Tauri 2.0
- **Frontend**: SvelteKit 5 with TypeScript
- **Styling**: Tailwind CSS 4
- **Math**: MathML for LaTeX rendering (pulldown-latex)
- **Markdown**: pulldown-cmark with extensions

### Project Structure

```
slideflare/
├── src/                   # SvelteKit frontend
├── src-tauri/             # Rust backend
├── static/                # Static assets
└── examples/              # Example presentations
```

### Development Commands

```bash
# Start development server
bun run tauri dev

# Type checking
bun run check

# Build production
bun run tauri build
```

### Contributing

We welcome contributions! Here's how you can help:

1. **🐛 Report bugs** via GitHub Issues
2. **💡 Suggest features** and improvements
3. **📝 Improve documentation**
4. **🔧 Submit pull requests**

Please read our contributing guidelines and code of conduct before contributing.

## 🔮 Roadmap

### Version 0.2.0
- [ ] **AI-powered slide conversion** from PowerPoint/PDF
- [ ] **Mermaid diagram support** for flowcharts and graphs
- [ ] **Shiki Magic Move** for animated code transitions
- [ ] **Custom themes** with JSON configuration

### Version 0.3.0
- [ ] **Multipart slides** for complex layouts
- [ ] **Performance optimizations** with parallel processing
- [ ] **PDF export** functionality
- [ ] **Online sharing** with generated links

### Future Versions
- [ ] **Collaborative editing** in real-time
- [ ] **Plugin system** for extensibility
- [ ] **Mobile app** for presentation control
- [ ] **Cloud synchronization**

## 📄 File Format

SlideFlare uses standard Markdown with YAML frontmatter:

```yaml
---
bg_color: bg-blue-600        # Tailwind background class
text_color: text-white       # Tailwind text class
title: Slide Title           # Optional slide title
layout: center               # Optional layout (center, default)
transition: fade             # Optional transition effect
---

# Your markdown content here

Regular markdown syntax with all the features you love.
```

## 🤝 Community

- **GitHub**: [MeatyAri/slideflare](https://github.com/MeatyAri/slideflare)
- **Issues**: [Report bugs or request features](https://github.com/MeatyAri/slideflare/issues)
- **Wiki**: [Documentation and guides](https://github.com/MeatyAri/slideflare/wiki)

## 📜 License

This project is licensed under the **Apache 2.0 License** - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/) for the cross-platform foundation
- Powered by [SvelteKit](https://kit.svelte.dev/) for the reactive frontend
- Styled with [Tailwind CSS](https://tailwindcss.com/) for rapid UI development
- Math rendering by [KaTeX](https://katex.org/) for beautiful equations
- Markdown parsing by [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)

---

<div align="center">

**Made with ❤️ and ☕ by [Meatyari](https://github.com/MeatyAri)**

*Star ⭐ this repo if you find SlideFlare useful!*

</div>
