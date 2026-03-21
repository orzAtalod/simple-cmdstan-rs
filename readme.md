# Simple-cmdstan-rs

simple-cmdstan-rs is a Rust interface for working with CmdStan.

The project is designed to simplify two parts of the CmdStan workflow:

1. assembling Stan-compatible input data in Rust
2. composing and executing CmdStan command arguments in a structured way

Rather than manually formatting JSON files and command-line arguments, this library provides typed abstractions for data preparation and command construction.

## Installation

Clone this repository and add it to your Cargo.toml as a local dependency:

``` toml
[dependencies]
simple-cmdstan-rs = { path = "path/to/simple-cmdstan-rs" }
```

Then import the prelude:

``` Rust
use simple_cmdstan_rs::prelude::*;
```

## Design overview

The library is organized around three main components:

+ DataCollection: assemble named Stan input data and export it as Stan-compatible JSON
+ argument trees (such as ArgSample and WithCommonArgs): build CmdStan command arguments as typed structures
+ arg_into: execute a compiled CmdStan model with a given argument tree

A typical workflow looks like this:

1. build input data with DataCollection
2. dump the data to a JSON file
3. create an argument tree for the CmdStan command
4. execute the model with arg_into

## DataCollection

DataCollection is a workflow-oriented container for assembling named Stan input data in Rust.

Internally, it stores a collection of named DataEntry values. Each entry corresponds to one field in the final Stan JSON input.

### Basic example

```Rust
let mut x = DataCollection::new();
x.add_entry("N", 5)
 .add_entry("vec", vec![2, 3, 2, 4, 2]);

assert_eq!(
    x.entries.write_as_stan_data(),
    "{\n    \"N\": 5,\n    \"vec\": [2, 3, 2, 4, 2]\n}"
);
```

### Complex values

You can use num::complex::Complex, or construct complex values explicitly with DataEntry::from_complex.

``` Rust
let mut x = DataCollection::new();
x.add_entry("N", 2)
    .add_entry("vec", vec![Complex::new(1, 2), Complex::new(3, 4)])
    .add_entry(
        "vec2",
        vec![
            DataEntry::from_complex(1.0, 2.0),
            DataEntry::from_complex(3.0, 4.0),
        ],
    );

assert_eq!(
    x.entries.write_as_stan_data(),
    "{\n    \"N\": 2,\n    \"vec\": [[1, 2], [3, 4]],\n    \"vec2\": [[1, 2], [3, 4]]\n}"
);
```

### Workflow-oriented helpers

DataCollection also supports incremental construction patterns that are useful in modelling workflows.

#### Build an array item by item

```Rust
let dcb = {
    let mut tmp = DataCollection::new();
    tmp.add_entry("vec", vec![1, 3, 2]);
    tmp
};

let dca = {
    let tmp = DataCollection::new();
    let mut tmp = tmp.open_array("vec");
    tmp.add_item(1).add_item(3).add_item(2);
    tmp.close_array()
};

assert_eq!(dca.entries.write_as_stan_data(), dcb.entries.write_as_stan_data());
```

#### Generate entries from a function

```Rust
let dca = {
    let mut tmp = DataCollection::new();
    tmp.add_entry_from_func("var", 3, || 1);
    tmp
};

let dcb = {
    let mut tmp = DataCollection::new();
    tmp.add_entry("var", vec![1, 1, 1]);
    tmp
};

assert_eq!(dca.entries.write_as_stan_data(), dcb.entries.write_as_stan_data());
```

### Exporting data

Use `.write_as_stan_data()` to serialize data into a string, or `dump_stan_json(...)` to write it directly to a JSON file.

```Rust
dump_stan_json(&x, "./bernoulli.json").unwrap();
```

## Argument trees and arg_into

The command side of the library is built around typed argument trees.

Instead of manually constructing CmdStan command-line arguments, you create structured argument objects and pass them to arg_into(...).

### Basic example

```rust
let sample = WithCommonArgs::new(
    ArgSample::new().with_num_samples(5000).unwrap()
)
.with_data("./bernoulli.json")
.unwrap();

arg_into(sample, "./model/bernoulli.exe");
```

In this setup:

+ `ArgSample` is the root command tree for sampling
+ `WithCommonArgs` attaches common CmdStan arguments such as data, random seed, and output options
+ `arg_into(...)` executes the compiled model with the given argument tree

### Nested argument trees

Argument trees can be nested. Invalid values return errors.

```Rust
let sample = ArgSample::new()
    .with_adapt(
        ArgSampleAdapt::new().with_kappa(0.8).unwrap()
    )
    .unwrap();
```

### Common arguments

WithCommonArgs is a wrapper that provides the common CmdStan arguments, such as:

+ data
+ random
+ init
+ output
+ id
+ num_threads

For example:

``` Rust
let mut sample = WithCommonArgs::new(
    ArgSample::new().with_num_samples(5000).unwrap()
)
.with_data("./bernoulli.json")
.unwrap();

sample.set_random(
    ArgRandom::new().with_seed(112233).unwrap()
);

```

### Return value of arg_into

arg_into returns a `StanResult<T>`, which stores:

+ a copy of the input argument tree
+ the output path
+ the `std::process::Output` produced by the CmdStan process

```Rust
pub struct StanResult<T: ArgThrough> {
    pub arg_tree: T,
    pub output_path: ArgReadablePath,
    pub output: std::process::Output,
}
```
