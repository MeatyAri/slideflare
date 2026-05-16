# TODO:

- [ ] add the cool intro example (make it prettier)
- [ ] publish the AI skill
- [ ] run tests on building release
- [ ] add easy to use fonts support
- [ ] add easy to use rtl support
- [ ] add multipart slides
- [ ] add mermaid diagrams
- [ ] add shiki magic move
- [ ] add themes, use JSON to create themes
- [ ] add better styling
- [ ] add easy way to convert slides from other platforms to slideflare:
  - [ ] get the pdf output of ther platforms and convert them to slideflare markdown using mistral OCR or other OCR tools that support images (mistral takes screenshots of the things that are not convertable to markdown)

- [ ] make slideflare faster: \
      either:
  - [ ] make the slide parsing more efficient by parsing everything at once rather than per slide \
        or:
  - [ ] use tokio instead of std for asynchronous file reading
  - [ ] parallelize the image/video processing OR slides processing as a whole

  - [ ] only process images/videos of the current slide or current + next slide (maybe remove previously loaded content after moving to the next slide)

- [ ] complete the documentation
- [ ] write more tests and move them into a separate folder
- [ ] Do not open links inside the app, open them in the browser, (ask for confirmation before opening the link)
- [ ] add to AUR

# Testing:

- [ ] test the examples provided in the readme
- [x] fix the `---` parsing problem

# Later

- [ ] add a help menu
- add convert to pdf
- add custom code component

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
