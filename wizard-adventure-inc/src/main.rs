use std::fs;

use anyhow::Result;

mod game;
mod tui;

fn main() -> Result<()> {
    let mut dot = false;
    let mut path = "game.rec".to_string();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--dot" => dot = true,
            other => path = other.to_string(),
        }
    }

    let raw = fs::read_to_string(&path)?;
    if dot {
        print_dot(&raw)
    } else {
        tui::run(&raw)
    }
}

/// Dump the full game graph as Graphviz dot, for debugging the data.
fn print_dot(raw: &str) -> Result<()> {
    let game = game::Game::from_rec_str(raw)?;
    println!("strict digraph {{");
    let mut pending = vec![game.start_id()];
    let mut seen = std::collections::HashSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        let node = game.node(id).expect("graph is validated at load");
        println!("  {} [label=\"{}: {}\"]", id, id, node.title);
        for &child in &node.connects_to {
            println!("  {} -> {}", id, child);
            pending.push(child);
        }
    }
    println!("}}");
    Ok(())
}
