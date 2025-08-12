# TODO:

- [ ] handle image/video/file paths correctly
- [ ] add a more permanent fix for the screen flashing and make sure the dark mode is getting handled properly
- [ ] add multipart slides
- [ ] add shiki magic move
- [ ] add themes, use JSON to create themes
- [ ] add AI
- [ ] check for bugs in the hashing system

# Testing:

- add [tailwindcss-typography](https://github.com/tailwindlabs/tailwindcss-typography) to style the html parsed from the markdown
- [x] add pulldown-cmark for markdown parsing
    - [x] enable all the extensions (enabled gfm, math, frontmatter(yaml support))
    - [x] add custom slides parsing
# Later

- use windicss or postcss to get rid of the tailwind.js file and remove the screen refreshing (it is done to get rid of the previously applied styles) (potential fixes)
- add convert to pdf
- add custom code component

# Done

- fixed file watcher and termination logic
- on resize scroll to the active slide with no animation (to keep it persistent when changing resolution or window size)
- make sure same file won't get processed twice, used a non-cryptographic hash function
