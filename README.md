# Mouse Mover

A simple program for warping the mouse between monitors on OSX.

You can use something like [skhd](https://github.com/koekeishiya/skhd) to bind this to hot keys like so:

```bash
# ~/.skhdrc

# Move focus between monitors using cmd + ][
cmd - 0x1E: /Users/innes/.cargo/bin/mouse-mover
cmd - 0x21: /Users/innes/.cargo/bin/mouse-mover --back
```
