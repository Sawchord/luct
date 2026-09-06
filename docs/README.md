# Documentation

This directory contains files used for the documentation of the tool.

## Website

The website is built using the static site generator [Zola](https://www.getzola.org/).

To install Zola, follow the [installation guide](https://www.getzola.org/documentation/getting-started/installation/)
on their web site.

Since you like have a rust toolchain already, probably the easiest is to run:

```
cargo install --locked --git https://github.com/getzola/zola
zola --version
```

Then to work on the content of the site, run:

```
zola serve
```

## Book

The tool is accompanied by a book, explaining it's capablilites.

To install `mdbook`, run:

```
cargo install mdbook mdbook-toc mdbook-katex mdbook-svgbob2 mdbook-linkcheck
```

Then to see changes you are making to the book run:

```
mdbook serve
```