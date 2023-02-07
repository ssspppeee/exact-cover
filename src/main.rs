use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
// use std::collections::{HashMap, HashSet};
use std::collections::HashMap;
use std::iter::zip;
use regex::Regex;

fn main () {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let s = Sudoku::read(file_path);

    println!("Problem:");
    println!("{:#?}", s);

    let ec = sudoku_to_exact_cover(&s);

    let mut solver = init_solver(&ec);

    let sol = solver.solve();

    println!("Solution raw:\n{sol:#?}");
    
    let t = s.read_exact_cover(ec, sol);

    println!("Solution:\n{t:#?}");
}

#[derive(Debug)]
struct Sudoku {
    grid: Vec<Vec<u8>>
}

impl Sudoku {
    fn read(file_path: &str) -> Self {
        return Self {
            grid: BufReader::new(fs::File::open(file_path).unwrap())
                .lines()
                .map(|row| row.unwrap().chars().collect::<Vec<_>>().iter()
                     .map(|digit| digit.to_digit(10).unwrap() as u8)
                     .collect())
                .collect()
        }
    }

    fn read_exact_cover(self, ec: ExactCover, s: ExactCoverSolution) -> Self {
        let mut sol_grid = self.grid.clone();
        let parser = Regex::new(r"\w(\d)(\d)").unwrap();
        for idx in s {
            let choice: Vec<Item>  = ec.choices[idx-1].iter().map(|i| ec.items[*i].clone()).collect();
            assert!(choice.len() == 4);
            let caps = parser.captures(&choice[0]).unwrap();
            let i = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
            let j = caps.get(2).unwrap().as_str().parse::<usize>().unwrap();
            let caps2 = parser.captures(&choice[1]).unwrap();
            let k = caps2.get(2).unwrap().as_str().parse::<u8>().unwrap();
            sol_grid[i][j] = k;
        }
        return Sudoku { grid: sol_grid };
    }
}

type Item = String;
// struct Choice<'a>(Vec<&'a Item>);
type Choice = Vec<usize>;
struct ExactCover {
    items:   Vec<Item>,
    choices: Vec<Choice>
}
struct ExactCoverSolver<'a> {
    ec: &'a ExactCover,
    name: Vec<String>,
    toplen: Vec<i32>,
    llink: Vec<i32>,
    rlink: Vec<i32>,
    ulink: Vec<i32>,
    dlink: Vec<i32>
}
type ExactCoverSolution = Vec<usize>;

fn init_solver<'a>(e: &'a ExactCover) -> ExactCoverSolver<'a> {
    let n_items = (e.items.len() - 1) as i32;
    let name2idx = zip(e.items.clone(), 0..=n_items).collect::<HashMap<_, _>>();

    let mut llink: Vec<i32> = (0..n_items).collect();
    llink.insert(0, n_items);

    let mut rlink: Vec<i32> = (1..=n_items).collect();
    rlink.push(0);

    let mut prev = zip(0..=n_items, 0..=n_items).collect::<HashMap<_, _>>();
    let mut count = n_items + 2;
    let mut ulink: Vec<i32> = vec![0; count as usize];
    let mut toplen: Vec<i32> = vec![0; count as usize];
    for (r, row) in e.choices.iter().enumerate() {
        let mut first_item = count;
        for (c, &val) in row.iter().enumerate() {
            toplen[val] += 1;
            toplen.push(val as i32);
            ulink.push(prev[&(val as i32)]);
            prev.insert(val as i32, count);
            count += 1;
        }
        toplen.push(-(r as i32) - 1);
        ulink.push(first_item);
        count += 1;
    }

    for i in 1..=n_items {
        ulink[i as usize] = prev[&i];
    }
    let mut dlink: Vec<i32> = vec![0; ulink.len()];
    for (i, val) in ulink.iter().enumerate() {
        if toplen[i] > 0 {
            dlink[ulink[i] as usize] = i as i32;
        }
    }
    let mut ptr = 0;
    for i in (0..ulink.len()).rev() {
        if toplen[i] <= 0 {
            dlink[i as usize] = ptr;
            ptr = (i as i32) - 1;
        }
    }
    return ExactCoverSolver { 
        ec: e,
        name: e.items.clone(),
        toplen: toplen,
        llink: llink, 
        rlink: rlink,
        ulink: ulink, 
        dlink: dlink
    };
}


impl ExactCoverSolver<'_> {
    fn solve(&mut self) -> ExactCoverSolution {
        // X1
        let N = self.llink.len() - 1;
        let Z = self.toplen.len() - 1;
        let mut l = 0;
        let mut x = vec![0; N];
        // X2
        while self.rlink[0] != 0 {
            // X3
            // seelct item with minimum remaining values
            let mut item = self.rlink[0];
            let mut minlen = self.toplen[item as usize];
            let mut i = item;
            while item != 0 {
                if self.toplen[item as usize] < minlen {
                    minlen = self.toplen[item as usize];
                    i = item;
                }
                item = self.rlink[item as usize];
            }
            // X4
            // prinln!("Covering {}", self.name[i as usize]);
            self.cover(i);
            x[l] = self.dlink[i as usize];
            // println!("l: {}\t: {:?}", l, x);
            // X5
            while x[l] == i {
                // X7
                // println!("Uncovering {}", self.name[i as usize]);
                self.uncover(i);
                // X8
                if l == 0 {
                    println!("Failed");
                    return vec![];
                }
                else {
                    l -= 1;
                    // X6
                    let mut p = x[l] - 1;
                    while p != x[l] {
                        let j = self.toplen[p as usize];
                        if j <= 0 {
                            p = self.dlink[p as usize];
                        }
                        else {
                            // println!("Uncovering {}", self.ec.name[j as usize]);
                            self.uncover(j);
                            p -= 1;
                        }
                    }
                    i = self.toplen[x[l] as usize];
                    x[l] = self.dlink[x[l] as usize];
                    // println("l: {}\t: {:?}", l, x);
                }
            }
            // X5 cont.
            
            let mut p = x[l] + 1;
            while p != x[l] {
                let j = self.toplen[p as usize];
                if j <= 0 {
                    p = self.ulink[p as usize];
                }
                else {
                    // println!("Covering {}", self.name[j as usize]);
                    self.cover(j);
                    p += 1;
                }
            }
            l += 1;
        }
        // get solution
        for i in 0..x.len() {
            while self.toplen[x[i] as usize] > 0 {
                x[i] += 1;
            }
            x[i] = -self.toplen[x[i] as usize];
        }
        return x[..l].iter().map(|&item| item as usize).collect::<Vec<usize>>();
    }

    fn cover(&mut self, i: i32) -> () {
        let mut p = self.dlink[i as usize];
        while p != i {
            self.hide(p);
            p = self.dlink[p as usize];
        }
        let l = self.llink[i as usize];
        let r = self.rlink[i as usize];
        self.rlink[l as usize] = r;
        self.llink[r as usize] = l;
    }

    fn hide(&mut self, p: i32) -> () {
        let mut q = p + 1;
        while q != p {
            let x = self.toplen[q as usize];
            let u = self.ulink[q as usize];
            let d = self.dlink[q as usize];
            if x <= 0 {
                q = u;
            }
            else {
                self.dlink[u as usize] = d;
                self.ulink[d as usize] = u;
                self.toplen[x as usize] -= 1;
                q += 1;
            }
        }
    }

    fn uncover(&mut self, i: i32) {
        let l = self.llink[i as usize];
        let r = self.rlink[i as usize];
        self.rlink[l as usize] = i;
        self.llink[r as usize] = i;
        let mut p = self.ulink[i as usize];
        while p != i {
            self.unhide(p);
            p = self.ulink[p as usize];
        }
    }

    fn unhide(&mut self, p: i32) {
        let mut q = p - 1;
        while q != p {
            let x = self.toplen[q as usize];
            let u = self.ulink[q as usize];
            let d = self.dlink[q as usize];
            if x <= 0 {
                q = d;
            }
            else {
                self.dlink[u as usize] = q;
                self.ulink[d as usize] = q;
                self.toplen[x as usize] += 1;
                q -= 1;
            }
        }
    }
}

fn sudoku_to_exact_cover(sudoku: &Sudoku) -> ExactCover {
    let mut exclude = HashMap::new();
    for i in 0..9 {
        for j in 0..9 {
            let k = sudoku.grid[i][j];
            let x = 3 * (i / 3) + (j / 3);
            if k != 0 {
                exclude.insert(format!("p{i}{j}"), true);
                exclude.insert(format!("r{i}{k}"), true);
                exclude.insert(format!("c{j}{k}"), true);
                exclude.insert(format!("b{x}{k}"), true);
            }
        }
    }
    let mut choices = Vec::new();
    for i in 0..9 {
        for j in 0..9 {
            for k in 1..=9 {
                let x = 3 * (i / 3) + (j / 3);
                if !exclude.contains_key(&format!("p{i}{j}")) &&
                    !exclude.contains_key(&format!("r{i}{k}")) &&
                    !exclude.contains_key(&format!("c{j}{k}")) &&
                    !exclude.contains_key(&format!("b{x}{k}")) {
                    choices.push(
                        vec![
                             format!("p{i}{j}"),
                             format!("r{i}{k}"),
                             format!("c{j}{k}"),
                             format!("b{x}{k}")
                        ]
                    );
                }
            }
        }
    }
    let mut names = HashMap::new();
    for row in &choices {
        for item in row {
            names.insert(item, true);
        }
    }
    let mut name = names.keys().cloned().cloned().collect::<Vec<Item>>();
    name.insert(0, String::from(""));
    let name2idx = zip(name.clone(), 0..name.len()).collect::<HashMap<_, _>>();
    println!("{:#?}", choices);
    return ExactCover {
        items: name,
        choices: choices.iter()
            .map(|row| row.iter()
                 .map(|val| name2idx[val])
                 .collect())
            .collect()
    }
}
