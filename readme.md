# Casino

Analyzing the behavior of a multi-armed bandit

## Build

Casino is a Rust simulation of various strategies for solving the multi-armed bandit problem (specifically a bernoulli bandit).

Build the Rust application and run it to create a CSV file of steps. Then use the Quarto journal to analyze them.

```sh
# Build
cargo build -r

# Run
./target/release/bandit 0.1 0.12 0.14 0.08 0.06 0.11 0.14 0.13 0.15 0.12 50000 epsilon 0.01 > bandits.csv
```

The Quarto journal has an invocation of the built simulation and is already hooked up to inspect it.