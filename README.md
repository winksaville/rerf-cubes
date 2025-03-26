# refr-cubes

This is cubes but I've refactored out create_text as
I'm going to create diameter on one side and on another
side of the cube will be the number when I create the 8
R_E_R_F cubes.


## Install

```
cargo install --path .
```
## Usage

```
$ cargo run -- --help
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running `target/debug/rerf-cubes --help`
Create one or more cubes with an optional tube in the center

Usage: rerf-cubes [OPTIONS] <LEN_SIDE>

Arguments:
  <LEN_SIDE>  

Options:
  -c, --cube-count <CUBE_COUNT>
          The number of cubes to create [default: 1]
  -m, --min-tube-diameter <MIN_TUBE_DIAMETER>
          The minimum diameter of the tube in mm, 0 for no tube [default: 0.0]
  -t, --tube-diameter-step <TUBE_DIAMETER_STEP>
          The number mm's to increase the tube diameter by when there are multiple cubes [default: 0.0]
  -s, --segments <SEGMENTS>
          The number of segments to use when creating the tube, minimum is 3 [default: 50]
  -h, --help
          Print help
  -V, --version
          Print version

```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
