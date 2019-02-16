# oriole-text-dev

This crate will compile font files into signed distance fields.
The produced texture can be used to render text on the GPU.
This shifts rendering fonts from runtime to compile time.
Currently, only the fixed set of glyphs specified 
at compile time can be displayed at runtime.

See [oriole-text](https://github.com/johannesvollmer/oriole-text) 
for more details.