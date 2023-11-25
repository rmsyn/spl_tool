# SPL tool (VisionFive2)

Port of the C implementation of StarFive's [`spl_tool`](https://github.com/starfive-tech/Tools/tree/master/spl_tool), which is originally written by [`strlcat`](https://github.com/strlcat).

**WARNING** This tool is in the very earliest stages of development. It still requires a test suite, fuzzing harnesses for file formats, and real-world testing.

## Usage

To use the CLI applicatiom, compile with the `cli` feature:

```
$ cd spl_tool
$ cargo run --features cli -- --file <path-to-spl-image> --create-spl-header
# To see a full list of options
$ cargo run --features cli -- --help
```

## Installation

The CLI application requires the `cli` feature:

```
$ cd spl_tool
$ cargo install --features cli --path .
```

## no-std compatibility

The library portion of `spl_tool` is `no-std` compatible by default, and can be used in embedded/bare-metal contexts.

## Alternatives

- `spl_tool` (C): <https://github.com/starfive-tech/Tools/tree/master/spl_tool>
- `vf2-header` (Rust): <https://github.com/jonirrings/vf2-header>

## License

`spl_tool` Rust is licensed under the same GPLv2+ license as the original C implementation.
