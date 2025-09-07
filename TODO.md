# TODO:

- [ ] handle image/video paths correctly
  - [x] image
  - [x] video
  - [ ] fix handling of absolute paths
  - [ ] use tokio instead of std
  - [ ] add proper styling to images/videos
  - [ ] only process images/videos of the current slide or current + next slide (maybe remove previously loaded content after moving to the next slide)
- [ ] parallelize the image/video processing OR slides processing as a whole
- [ ] add multipart slides
- [ ] add mermaid diagrams
- [ ] add shiki magic move
- [ ] add themes, use JSON to create themes
- [ ] add AI
- [ ] make the slide parsing more efficient by parsing everything at once rather than per slide
- [ ] add better styling
- [ ] add easy way to convert slides from other platforms to slideflare:
  - [ ] get the pdf output of ther platforms and convert them to slideflare markdown using mistral OCR or other OCR tools that support images (mistral takes screenshots of the things that are not convertable to markdown)

# Testing:

# Later

- use windicss or postcss to get rid of the tailwind.js file and remove the screen refreshing (it is done to get rid of the previously applied styles) (potential fixes)
- add convert to pdf
- add custom code component

# Done

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
