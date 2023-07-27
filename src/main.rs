use std::cmp::Ordering;
use std::env;
use std::fmt::{Display, Write};

use rand::prelude::*;

const NUM_BANDITS: usize = 10;

#[derive(Debug)]
enum Strategy {
    EpsilonGreedy(f64),
    NaiveRandom,
    ConstantFirst,
}

impl Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Strategy::EpsilonGreedy(e) => write!(f, "Epsilon Greedy, e = {}", e),
            Strategy::NaiveRandom => write!(f, "NaiveRandom"),
            Strategy::ConstantFirst => write!(f, "ConstantFirst"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Bandit {
    plays: i64,
    wins: i64,
    prob_real: f64,
    prob_est: f64,
}

impl PartialOrd for Bandit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.prob_est < other.prob_est {
            Some(Ordering::Less)
        } else if self.prob_est > other.prob_est {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Bandit {
    fn new(prob_real: f64) -> Bandit {
        Bandit {
            plays: 0,
            wins: 0,
            prob_real,
            prob_est: 0.5,
        }
    }
}

impl Display for Bandit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{:.3}",
            self.plays, self.wins, self.prob_real, self.prob_est
        )
    }
}

#[derive(Debug)]
struct Casino {
    num_plays: i64,
    strategy: Strategy,
    bandits: [Bandit; NUM_BANDITS],
}

impl Casino {
    fn new(num_plays: i64, strategy: Strategy, bandits: [Bandit; NUM_BANDITS]) -> Casino {
        Casino {
            num_plays,
            strategy,
            bandits,
        }
    }

    fn play(&mut self) {
        let mut rng = rand::thread_rng();

        let headers = (1..=NUM_BANDITS)
            .map(|i| format!("bandit_{}", i))
            .collect::<Vec<String>>()
            .join(",");

        println!("{headers},regret");

        for _ in 1..self.num_plays {
            let snapshot = self
                .bandits
                .iter()
                .map(|b| format!("{:.5}", b.prob_est))
                .collect::<Vec<String>>()
                .join(",");

            let mut bandit = match self.strategy {
                Strategy::EpsilonGreedy(e) => {
                    pick_bandit_epsilon_greedy(&mut rng, &mut self.bandits, e)
                }
                Strategy::NaiveRandom => pick_bandit_naive_random(&mut rng, &mut self.bandits),
                Strategy::ConstantFirst => pick_first_bandit_always(&mut self.bandits),
            };

            let roll: f64 = rng.gen();

            bandit.plays += 1;

            if roll <= bandit.prob_real {
                bandit.wins += 1;
            }

            bandit.prob_est = (bandit.wins as f64) / (bandit.plays as f64);

            println!("{snapshot},{}", self.regret());
        }
    }

    fn regret(&self) -> f64 {
        let ideal_bandit = self
            .bandits
            .iter()
            .max_by(|a, b| compare_bandits_by_p_real(a, b))
            .unwrap();

        let ideal_bandit_true_p = ideal_bandit.prob_real;
        let plays_so_far = self.bandits.iter().fold(0.0, |c, a| c + a.plays as f64);

        let ideal = ideal_bandit_true_p * plays_so_far;
        let real = self.bandits.iter().fold(0.0, |c, a| c + a.wins as f64);

        ideal - real
    }
}

impl Display for Casino {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();

        writeln!(&mut buf, "{} - {} Plays", self.strategy, self.num_plays)?;
        writeln!(&mut buf, "Bandit ID\tPlays\tWins\tP(real)\tP(est)")?;

        for (idx, bandit) in self.bandits.iter().enumerate() {
            writeln!(&mut buf, "Bandit #{}\t{bandit}", idx + 1)?;
        }

        writeln!(&mut buf, "Total Regret: {}", self.regret())?;

        write!(f, "{buf}")
    }
}

fn compare_bandits_by_p_real(bandit_a: &Bandit, bandit_b: &Bandit) -> Ordering {
    if bandit_a.prob_real < bandit_b.prob_real {
        Ordering::Less
    } else if bandit_a.prob_real > bandit_b.prob_real {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn compare_bandits_by_p_est(bandit_a: &Bandit, bandit_b: &Bandit) -> Ordering {
    if bandit_a.prob_est < bandit_b.prob_est {
        Ordering::Less
    } else if bandit_a.prob_est > bandit_b.prob_est {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn pick_first_bandit_always(bandits: &mut [Bandit; NUM_BANDITS]) -> &mut Bandit {
    &mut bandits[0]
}

fn pick_bandit_epsilon_greedy<'rng, 'ban>(
    rng: &'rng mut ThreadRng,
    bandits: &'ban mut [Bandit],
    epsilon_not: f64,
) -> &'ban mut Bandit {
    let bandit: &mut Bandit;
    let epsilon: f64 = rng.gen();

    if epsilon <= epsilon_not {
        // Exploration round
        // The bandit will be one chosen at random
        let r_idx = rng.gen_range(0..NUM_BANDITS);

        bandit = &mut bandits[r_idx];
    } else {
        // Greed round
        // The bandit to play will be the one with the highest known probability
        bandit = &mut *bandits
            .iter_mut()
            .max_by(|a, b| compare_bandits_by_p_est(a, b))
            .unwrap();
    }

    bandit
}

fn pick_bandit_naive_random<'rng, 'ban>(
    rng: &'rng mut ThreadRng,
    bandits: &'ban mut [Bandit],
) -> &'ban mut Bandit {
    let r_idx = rng.gen_range(0..NUM_BANDITS);
    &mut bandits[r_idx]
}

fn run_casino_with_params(num_plays: i64, probs: [f64; NUM_BANDITS], strategy: Strategy) -> () {
    let bandits: [Bandit; NUM_BANDITS] = probs
        .iter()
        .map(|p| Bandit::new(*p))
        .collect::<Vec<Bandit>>()
        .try_into()
        .unwrap();

    let mut casino = Casino::new(num_plays, strategy, bandits);

    casino.play();
}

fn main() {
    let args = &mut env::args();
    let mut probs: Vec<f64> = vec![];

    let _bin_name = args.next().unwrap();

    for idx in 1..=NUM_BANDITS {
        let p = args
            .next()
            .and_then(|p| p.parse::<f64>().ok())
            .expect(format!("Arg {} must be of type f64", idx).as_str());

        probs.push(p);
    }

    let num_plays = args
        .next()
        .and_then(|p| p.parse::<i64>().ok())
        .expect(format!("Arg {} must be of type i64", NUM_BANDITS + 1).as_str());

    let strategy = match args
        .next()
        .expect(format!("Arg {} must be `epsilon n` or `naive`", NUM_BANDITS + 2).as_str())
        .as_str()
    {
        "epsilon" => {
            let epsilon = args
                .next()
                .and_then(|e| e.parse::<f64>().ok())
                .expect("Epsilon strategy requires e value of type f64");
            Strategy::EpsilonGreedy(epsilon)
        }
        "naive" => Strategy::NaiveRandom,
        "constant" => Strategy::ConstantFirst,
        _ => panic!("Final arg must be one of (epsilon n, naive, constant)"),
    };

    run_casino_with_params(num_plays, probs.try_into().unwrap(), strategy);
}
