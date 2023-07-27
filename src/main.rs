use std::cmp::Ordering;
use std::env;
use std::fmt::{Display, Write};

use rand::prelude::*;

const NUM_BANDITS: usize = 10;

#[derive(Debug)]
enum Strategy {
    EpsilonGreedy(f64),
    NaiveRandom,
}

impl Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Strategy::EpsilonGreedy(e) => write!(f, "Epsilon Greedy, e = {}", e),
            Strategy::NaiveRandom => write!(f, "NaiveRandom"),
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
        if self.prob_real < other.prob_real {
            Some(Ordering::Less)
        } else if self.prob_real > other.prob_real {
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

    fn regret(&self) -> i64 {
        let ideal_bandit = self
            .bandits
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let ideal_bandit_true_p = ideal_bandit.prob_real;
        let plays_so_far = self.bandits.iter().fold(0 as i64, |c, a| c + a.plays);

        let ideal = (ideal_bandit_true_p * (plays_so_far as f64)).round() as i64;
        let real = self.bandits.iter().fold(0 as i64, |c, a| c + a.wins);

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
            .max_by(|a, b| a.partial_cmp(b).unwrap())
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
        _ => panic!("Final arg must be a strategy"),
    };

    run_casino_with_params(num_plays, probs.try_into().unwrap(), strategy);
}