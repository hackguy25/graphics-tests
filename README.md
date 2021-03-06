# Graphics tests
A collection of programs I wrote to familiarize myself with computer graphics in Rust.

## `pixels`

A collection of programs using the [`pixels`](https://lib.rs/crates/pixels) 2D frame buffer.

### Computer Graphics from Scratch

Modules `cgfs_raytracing`, `cgfs_rasterization` and `cgfs_scene` contain code described in the book [Computer Graphics from Scratch](https://gabrielgambetta.com/computer-graphics-from-scratch/).
The code follows the examples in book mostly faithfully, although there are some tweaks here and there, either due to programming language differences or simply as performance improvements.
Module `cgfs_raytracing` corresponds to chapters 2 through 4, `cgfs_rasterization` to chapters 6 through 9, and finally `cgfs_scene` to chapters 10 through 12.
Due to the complexity of the final rasterizer I've decided not to implement shading and textures.

### Mandelbrot set

Module `mandel` contains code used to generate and display the [Mandelbrot set](https://en.wikipedia.org/wiki/Mandelbrot_set).

## `glfw`

A simple example using the [`glfw`](https://lib.rs/crates/glfw) library.
A work in progress with no real goal.

## `rainbow-smoke`

An implementation of the rainbow smoke algorithm, as presented in the accepted answer of the question [Images with all colors](https://codegolf.stackexchange.com/questions/22144/images-with-all-colors).
Uses `pixels` library as a base, iteratively updates the canvas.