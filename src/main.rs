use std::cmp::min;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;

type Color = u32;
type Column = Vec<Color>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Score {
    Score(usize),
    Win,
}

#[derive(Debug, Clone)]
struct Puzzle {
    column_size: usize,
    colors_count: HashMap<Color, usize>,
    state: Vec<Column>,
}

impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.column_size {
            if i > 0 {
                writeln!(f)?;
            }
            for j in 0..self.state.len() {
                let col = &self.state[j];
                if j > 0 {
                    write!(f, " ")?;
                }
                let idx = self.column_size - i - 1;
                let c = col
                    .get(idx)
                    // This is pretty bad since it will only print something meaningful if callers
                    // passed values from 0 to 9 in the columns, but this is just toy code anyways.
                    .map(|&x| char::from_digit(x, 10).unwrap_or('?'))
                    .unwrap_or(' ');
                write!(f, "[{c}]")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Move(usize, usize);

impl Puzzle {
    fn new(column_size: usize, init: &[Vec<u32>]) -> Self {
        let mut colors_count = HashMap::new();
        let mut state = Vec::new();

        for col in init {
            let mut vec = Vec::with_capacity(column_size);
            for &c in &col[..min(column_size, col.len())] {
                let entry = colors_count.entry(c).or_insert(0);
                *entry += 1;
                vec.push(c);
            }
            state.push(vec);
        }

        Puzzle {
            column_size,
            colors_count,
            state,
        }
    }

    fn rank(&self) -> Score {
        let mut score: usize = 0;
        let mut done = true;

        for (i, col) in self.state.iter().enumerate() {
            // Adding the number of moves to the score to promote states that are not stuck.
            score += self.column_moves(i).count();
            // We use self.state.len() as a multiplier to ensure the various conditions below
            // (empty columns, columns with just one color, columns fully sorted with all the
            // entries of that color) dominate over just being able to move items.
            if let Some(&c) = col.last() {
                if col.iter().all(|&c2| c2 == c) {
                    // Column containing just a single color
                    if col.len() == self.colors_count[&c] {
                        // Column with all the entries of a single color
                        score += 1000 * self.state.len();
                    } else {
                        score += 100 * self.state.len();
                        done = false;
                    }
                } else {
                    done = false;
                }
            } else {
                // Empty column
                score += 10 * self.state.len();
            }
        }

        if done {
            return Score::Win;
        }
        Score::Score(score)
    }

    fn column_moves(&self, col: usize) -> impl Iterator<Item = Move> {
        let src = &self.state[col];
        let iter;

        if let Some(&c) = src.last() {
            iter = Some(
                self.state
                    .iter()
                    .enumerate()
                    .filter(move |(i, _)| *i != col)
                    .filter(move |(_, dst)| dst.last().is_none_or(|&c2| c2 == c))
                    .filter(|(_, dst)| dst.len() < self.column_size)
                    .map(move |(i, _)| Move(col, i)),
            );
        } else {
            iter = None;
        }

        iter.into_iter().flatten()
    }

    fn moves(&self) -> impl Iterator<Item = Move> {
        self.state
            .iter()
            .enumerate()
            .flat_map(|(i, _)| self.column_moves(i))
    }

    fn do_move(&mut self, Move(from, to): Move) {
        let &color = self.state[from]
            .last()
            .expect("cannot move from an empty column");

        while self.state[to].len() < self.column_size
            && let Some(c) = self.state[from].pop_if(|c2| *c2 == color)
        {
            self.state[to].push(c);
        }
    }

    fn dfs(&self, depth: u32, score: Score) -> (Score, VecDeque<Move>) {
        if depth == 0 {
            return (score, VecDeque::new());
        }

        // Evaluate all nodes at the given depth
        let mut best_score = score;
        let mut best_moves = VecDeque::new();
        for m in self.moves() {
            let mut game = self.clone();
            game.do_move(m);
            let (child_score, mut moves) = game.dfs(depth - 1, game.rank());
            if child_score > best_score {
                best_score = child_score;
                moves.push_front(m);
                best_moves = moves;

                if let Score::Win = child_score {
                    break;
                }
            }
        }

        (best_score, best_moves)
    }

    // IDDFS
    fn solve(&self, max_depth: u32, iterations: u32) -> VecDeque<Move> {
        let mut all_moves = VecDeque::new();
        let mut count = 0;
        let mut game = self.clone();
        while count < iterations {
            let mut best_moves = VecDeque::new();
            for d in 0..max_depth {
                let (score, moves) = game.dfs(d, game.rank());
                if let Score::Win = score {
                    all_moves.extend(moves);
                    println!("Found a winner in {} moves.", all_moves.len());
                    return all_moves;
                }
                best_moves = moves;
            }
            for m in &best_moves {
                game.do_move(*m);
            }
            all_moves.extend(best_moves);
            count += 1;
        }
        all_moves
    }
}

fn main() {
    let mut p = Puzzle::new(
        4,
        &[
            vec![1, 2, 3, 4],
            vec![3, 5, 3, 1],
            vec![6, 1, 2, 5],
            vec![6, 3, 2, 5],
            vec![6, 5, 4, 6],
            vec![2, 1, 4, 4],
            vec![],
            vec![],
        ],
    );
    let moves = p.solve(5, 100);
    println!("Initial state:\n{p}");
    for m in moves {
        print!("{m:?} -> ");
        p.do_move(m);
        println!("{:?}", p.rank());
        println!("{p}");
    }
}
