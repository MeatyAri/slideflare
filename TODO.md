# TODO:

- [ ] fix screen resizing issue
- [ ] add a back button in UI + esc as shorcut
- [ ] add a tutorial that pops op on the first use and on every update when new features are added
- [ ] add better default styling
  - [ ] to heading tags
- [ ] add the cool intro example (make it prettier)
- [ ] publish the AI skill
  - [ ] mention it in the readme
- [ ] add security implications to prevent a melicious slide deck from doing XSS and other types of attacks
  - [ ] use ammonia
- [ ] run tests on building release
- [ ] add easy to use fonts support
- [ ] add easy to use rtl support
- [ ] add shiki magic move
- [ ] add code syntax highlighting
- [ ] add mermaid diagrams
- [ ] add multipart slides
- [ ] add themes, use JSON to create themes
- [ ] add easy way to convert slides from other platforms to slideflare:
  - [ ] get the pdf output of ther platforms and convert them to slideflare markdown using mistral OCR or other OCR tools that support images (mistral takes screenshots of the things that are not convertable to markdown)

- [ ] make slideflare faster: \
  - [ ] pase everything at once rather when rendering whole file and parse per slide when doing incremental updates \
  - [ ] use tokio instead for asynchronous media processing
  - [ ] only process images/videos of the current slide or current + next slide (maybe remove previously loaded content after moving to the next slide)

- [ ] complete the documentation
- [ ] write more tests and move them into a separate folder
- [ ] Do not open links inside the app, open them in the browser, (ask for confirmation before opening the link)

# Testing:

- [ ] test the examples provided in the readme
- [x] add to AUR
- [x] the reload button should reread the file and do the whole parsing pipeline assuming that something went wrong

# Later

- [ ] add a help menu
- [ ] add convert to pdf
- [ ] add custom code component

# Done

- [x] fix the lowercase/capital s on the program title
- [x] write a better readme
- [x] fix the licensing
- add [tailwindcss-typography](https://github.com/tailwindlabs/tailwindcss-typography) to style the html parsed from the markdown
- [x] add pulldown-cmark for markdown parsing
  - [x] enable all the extensions (enabled gfm, math, frontmatter(yaml support))
  - [x] add custom slides parsing
- [x] check for bugs in the hashing system
  - there were actually no bugs in the hashing system
  - the problem was with the screen refreshing after any slide change
- [x] add a more permanent fix for the screen flashing and make sure the dark mode is getting handled properly
- fixed file watcher and termination logic
- on resize scroll to the active slide with no animation (to keep it persistent when changing resolution or window size)
- make sure same file won't get processed twice, used a non-cryptographic hash function
- [x] handle image/video paths correctly
  - [x] image
  - [x] video
  - [x] fix handling of absolute paths
  - [x] add proper styling to images/videos
- [x] Update the incremental.rs tests
- [x] replace temp incremental.rs implementations with imara-diff
- [x] Performance Bug (watcher.rs:66 & 94): compute_slide_hashes() called twice, causing unnecessary double parsing
- [x] fix jumping on the first slide after an edit
  - use the hash to know what the correct slide is
- [x] add an error screen for when the syntax is not correct
  - [x] make sure it displays the error when opening new slides
  - [x] make sure it displays an error pop up on the opened slide when editing
  - [x] fix: the new slide validator is detecting an error for the perfectly fine example in the example.md
- [x] markdown next to a html line won't get detected (check if there's a possible fix)
  - not possible, it's a CommonMark spec
- [x] update the readme, verify links work and remove katex from acknowledgments
  - [x] mention the AUR installation
- [x] fix the `---` parsing problem
- [x] find a solution for white or close to white backgrounds that make the text inisible
  - [x] the text color property is not getting applied to the heading tags
