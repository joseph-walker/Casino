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

## Changing the Number of Bandits

The simulation uses an array, whose size must be statically known at compile time. It is controlled through the `NUM_BANDITS` const - change this value to update the number of choices in the simulation.

## Example

<img width="600" alt="image" src="https://github.com/joseph-walker/Casino/assets/14129003/804b7d37-876e-45e9-96f0-94cad485343e" />

<img width="600" alt="image" src="https://github.com/joseph-walker/Casino/assets/14129003/f2a6123f-c1b3-4946-8283-a05c964c47be">
