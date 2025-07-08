use std::cmp::Ordering;
use std::cmp::min;
use std::collections::HashMap;
use std::fmt;

type Color = u32;
type Column = Vec<Color>;

#[derive(Debug, PartialEq, Eq)]
enum Score {
    Win,
    Score(usize),
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Score::Win, _) => Ordering::Greater,
            (_, Score::Win) => Ordering::Less,
            (Score::Score(s1), Score::Score(s2)) => s1.cmp(s2),
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct Puzzle {
    column_size: usize,
    colors_count: HashMap<Color, usize>,
    state: Vec<Column>,
}

impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
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

#[derive(Debug)]
struct MoveTree {
    game: Puzzle,
    children: HashMap<Move, MoveTree>,
}

impl Puzzle {
    fn new(column_size: usize, init: &[Vec<u32>]) -> Puzzle {
        let mut p = Puzzle {
            column_size,
            colors_count: HashMap::new(),
            state: Vec::new(),
        };

        for col in init {
            let mut vec = Vec::with_capacity(column_size);
            for &c in &col[..min(column_size, col.len())] {
                let entry = p.colors_count.entry(c).or_insert(0);
                *entry += 1;
                vec.push(c);
            }
            p.state.push(vec);
        }
        p
    }

    fn rank(&self) -> Score {
        let mut score: usize = 0;
        let mut done = true;

        for (i, col) in self.state.iter().enumerate() {
            // Adding the number of moves to the score to promote states that are not stuck.
            score += self.column_moves(i).len();
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

    fn column_moves(&self, col: usize) -> Vec<Move> {
        let mut moves = Vec::new();
        let src = &self.state[col];

        let Some(&c) = src.last() else {
            return Vec::new();
        };

        for (i, dst) in self.state.iter().enumerate() {
            if i == col {
                continue;
            }
            if dst.last().is_some_and(|&c2| c2 != c) {
                continue;
            }
            if dst.len() < self.column_size {
                moves.push(Move(col, i))
            }
        }
        moves
    }

    fn moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        for i in 0..self.state.len() {
            moves.extend(&self.column_moves(i));
        }
        moves
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

    fn moves_tree(&self, depth: u32) -> MoveTree {
        MoveTree {
            game: self.clone(),
            children: self.moves_map(depth),
        }
    }

    fn moves_map(&self, depth: u32) -> HashMap<Move, MoveTree> {
        if depth == 0 {
            return HashMap::new();
        }

        let mut children = HashMap::new();

        for &m in &self.moves() {
            let mut game = self.clone();
            game.do_move(m);
            let map = game.moves_map(depth - 1);
            children.insert(
                m,
                MoveTree {
                    game,
                    children: map,
                },
            );
        }
        children
    }

    fn solve(&self, depth: u32, iterations: u32) -> Vec<Move> {
        let mut count = 0;
        let mut game = &self.clone();
        let mut moves = Vec::new();
        let mut tree;
        while count < iterations {
            tree = game.moves_tree(depth);
            let (new_game, score, next_moves) = tree.find_best();
            moves.extend(&next_moves);

            if let Score::Win = score {
                break;
            }
            game = new_game;
            count += 1;
        }
        moves
    }
}

impl MoveTree {
    fn find_best(&self) -> (&Puzzle, Score, Vec<Move>) {
        let game = &self.game;
        let score = game.rank();

        // Using matches!() here because you cannot group if let with another condition
        // using the || operator (although it is allowed with &&).
        if matches!(score, Score::Win) || self.children.is_empty() {
            return (game, score, Vec::new());
        }

        let mut best_score = Score::Score(0);
        let mut best_moves = Vec::new();
        let mut best_game = game;

        for (&m, tree) in &self.children {
            let (new_game, score, moves) = tree.find_best();
            if score > best_score {
                best_score = score;
                best_moves = vec![m];
                best_moves.extend(&moves);
                best_game = new_game;
            }
            if let Score::Win = best_score {
                break;
            }
        }
        (best_game, best_score, best_moves)
    }
}

fn main() {
    let mut p = Puzzle::new(
        4,
        &[vec![1, 2, 3, 4], vec![1, 2, 3, 4], vec![], vec![], vec![]],
    );
    let moves = p.solve(5, 100);
    println!("Initial state: {p}");
    for m in moves {
        println!("{m:?}");
        p.do_move(m);
        println!("-> {p}");
    }
}
