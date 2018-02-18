# vgrapher

vgrapher is a 3D graphing calculator written in Rust that uses Vulkan as a
rendering backend (through vulkano).

Here's a somewhat underwhelming screenshot:

![screenshot](https://i.imgur.com/1ozXnVc.png)

The blue cube in the center is the origin. The equation being graphed is
`y = (x * z) / 4`. It's not very pleasant to look at due to the lack of shading,
but it's slightly less gross in the actual application where you can actually
move around.

### Controls

WASD moves around, space goes up, left shift goes down, and the mouse looks around.
To add an equation to the graph list press `Enter`. The screen will freeze and you
can then enter the expression in the terminal. You can use the variables `x` and `z`
in the expression, which is evaluated to get `y`. As of right now, you can only do
basic operations (+ - * / ^) in the expression, and to use negatives you have to enter
`(0-n)` or the parsing of the equation will fail (I could just use meval, but this
project actually started as an attempt to write my own equation parser). This
functionality is planned to be implemented soon. To clear the equations list,
press right shift.

### Disclaimer

I am totally inexperienced with graphics programming, and this project definitely
contains some errors. The entirety of the `vrender` dependency is basically a fancy
version of the vulkano triangle example. The implementation in this repository is also
quite inefficient due to the use of Vectors instead of slices and other stuff. It's
also currently recalculating the equation every frame.