# simple-cmdstan-rs

An interface with CmdStan.

## Initialize

**Important!**

use stan_init function to change the current directory to your StanHome, or the `make` command won't be excuted properly.

``` Rust
stan_init("D:\\Anaconda3\\envs\\stan\\Library\\bin\\cmdstan");
```

## DataCollection

A DataCollection struct could operate in many ways, and could be written into a .json file which CmdStan accepts.

``` Rust
todo!();
```

## StanModel

Create a StanModel with a .stan file and its path (absolute or relative to StanHome)

``` Rust
// They're all OK and result same
let model_uncomplied_1 = StanModel::new("examples\\bernoulli\\", "bernoulli.stan");
let model_uncomplied_2 = StanModel::new("examples\\bernoulli", "bernoulli");
// This will create a complied model
let model_complied = StanModel::new("examples\\bernoulli", "bernoulli.exe");
// Use complie method to complie a model
model_uncomplied_1.complie().unwrap();

// Link the model with data
model_uncomplied_1.link_data(/*Any DataCollection you created*/);
// Any form of data that impl StanData trait is accepted!
model_uncomplied_2.link_data(('N',"y",/*Vector that contains the data*/));
// Use write_stan_data method to dump the data
model_uncomplied_1.write_stan_data();
// ..Or directly link it to an already exists data
model_complied.set_data_path("examples\\bernoulli\\bernoulli.data.json");
```

## StanCommand

A StanCommand struct is a uniformed platform to create formatted stan command.

Currently it's not smart enough to auto-complete the argument and figure out the bad argument :(. (It does need a lot of work!)

## TODO

- [ ] Add a method to insert vec<vec<T\>\> into DataCollection without a large number of copies.
- [ ] The type of path is String, but it needs to be replaced with std::path::Path to achieve better cross-platform consistency.
- [ ] The command is based on Windows, it may needs to be modified to support Linux and Mac.
- [ ] Add parallel
- [ ] Add more commands (like diagnose toolset)
