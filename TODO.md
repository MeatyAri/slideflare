# TODO:

- add pulldown-cmark for markdown parsing
    - enable all the extensions (enabled gfm, math, frontmatter(yaml support))
    - add custom slides parsing

# Testing:

- make sure same file won't get processed twice, used a non-cryptographic hash function
- add [tailwindcss-typography](https://github.com/tailwindlabs/tailwindcss-typography) to style the html parsed from the markdown

# Later

- add custom code component

# Done

- fixed file watcher and termination logic
- on resize scroll to the active slide with no animation (to keep it persistent when changing resolution or window size)
