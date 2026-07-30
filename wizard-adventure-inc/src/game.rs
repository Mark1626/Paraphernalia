//! Game engine: loads a game graph from a recfile and runs it.
//!
//! No UI or wall-clock dependencies; the frontend drives time by calling
//! [`Game::tick`] with elapsed seconds and renders from the query methods.
//!
//! Parts of the API aren't used by the TUI yet but are meant for extraction
//! into a standalone framework crate, hence the dead_code allowance.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use anyhow::{Context, Result, anyhow};
use recutils_rs::RecordRef;

pub const STATE_PLAYING: &str = "playing";
pub const STATE_OUT_OF_TIME: &str = "out-of-time";

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub log: Option<String>,
    pub connects_to: Vec<u32>,
    pub requirements: Vec<String>,
    pub add_to_inventory: Vec<(String, u32)>,
    pub remove_from_inventory: Vec<(String, u32)>,
    pub state_change_to: Option<String>,
    pub time_to_complete: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Available,
    /// Discovered but requirements are not in the inventory yet.
    Locked,
    InProgress,
    Completed,
}

#[derive(Debug, Clone)]
pub enum Event {
    Log { node_id: u32, text: String },
    Discovered(Vec<u32>),
    ItemAdded { item: String, quantity: u32 },
    ItemRemoved { item: String, quantity: u32 },
    StateChanged(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    GameOver,
    Busy,
    NotDiscovered,
    AlreadyCompleted,
    MissingItems(Vec<String>),
    UnknownNode(u32),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::GameOver => write!(f, "the game is over"),
            ActionError::Busy => write!(f, "another action is in progress"),
            ActionError::NotDiscovered => write!(f, "that action has not been discovered"),
            ActionError::AlreadyCompleted => write!(f, "that action is already done"),
            ActionError::MissingItems(items) => write!(f, "requires: {}", items.join(", ")),
            ActionError::UnknownNode(id) => write!(f, "no such action: {id}"),
        }
    }
}

struct ActiveAction {
    id: u32,
    remaining: f64,
}

pub struct Game {
    nodes: BTreeMap<u32, Node>,
    start: u32,
    /// Discovery order, so the UI gets a stable listing.
    discovered: Vec<u32>,
    completed: BTreeSet<u32>,
    /// Item name to quantity held; entries are removed when they hit zero.
    inventory: BTreeMap<String, u32>,
    active: Option<ActiveAction>,
    time_used: f64,
    max_time: f64,
    state: String,
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Pair a comma-separated item list with its comma-separated quantity list.
/// Quantities align by position and default to 1 when absent.
fn parse_item_list(
    id: u32,
    items: Option<String>,
    quantities: Option<String>,
    qty_field: &str,
) -> Result<Vec<(String, u32)>> {
    let items = items.map(|v| split_list(&v)).unwrap_or_default();
    let quantities = quantities
        .map(|v| {
            split_list(&v)
                .iter()
                .map(|q| {
                    q.parse::<u32>()
                        .ok()
                        .filter(|&q| q > 0)
                        .ok_or_else(|| anyhow!("node {id}: bad {qty_field} entry {q:?}"))
                })
                .collect::<Result<Vec<u32>>>()
        })
        .transpose()?
        .unwrap_or_default();
    if quantities.len() > items.len() {
        return Err(anyhow!("node {id}: more {qty_field} entries than items"));
    }
    Ok(items
        .into_iter()
        .enumerate()
        .map(|(i, item)| (item, quantities.get(i).copied().unwrap_or(1)))
        .collect())
}

fn parse_node(record: &RecordRef<'_>) -> Result<Node> {
    let id: u32 = record
        .get_field("id")
        .ok_or_else(|| anyhow!("record missing id"))?
        .trim()
        .parse()
        .context("id is not a number")?;
    let field = |name: &str| record.get_field(name);
    let title = field("title").ok_or_else(|| anyhow!("node {id} missing title"))?;
    let time_to_complete: f64 = field("time_to_complete")
        .ok_or_else(|| anyhow!("node {id} missing time_to_complete"))?
        .trim()
        .parse()
        .with_context(|| format!("node {id}: time_to_complete is not a number"))?;
    let connects_to = field("connects_to")
        .map(|v| {
            split_list(&v)
                .iter()
                .map(|c| {
                    c.parse::<u32>()
                        .with_context(|| format!("node {id}: bad connects_to entry {c:?}"))
                })
                .collect::<Result<Vec<u32>>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(Node {
        id,
        title,
        description: field("description").filter(|d| !d.trim().is_empty()),
        log: field("log"),
        connects_to,
        requirements: field("requirement").map(|v| split_list(&v)).unwrap_or_default(),
        add_to_inventory: parse_item_list(
            id,
            field("add_to_inventory"),
            field("add_quantity"),
            "add_quantity",
        )?,
        remove_from_inventory: parse_item_list(
            id,
            field("remove_from_inventory"),
            field("remove_quantity"),
            "remove_quantity",
        )?,
        state_change_to: field("state_change_to"),
        time_to_complete,
    })
}

impl Game {
    pub fn from_rec_str(text: &str) -> Result<Game> {
        let mut db = recutils_rs::Db::parse_str(text).map_err(|e| anyhow!("{e}"))?;

        let max_time = match db.rset_by_type("GameMetadata") {
            Some(meta) => match meta.records().next().and_then(|r| r.get_field("max_time")) {
                Some(v) => v.trim().parse().context("max_time is not a number")?,
                None => f64::INFINITY,
            },
            None => f64::INFINITY,
        };

        let rset = db
            .rset_by_type("GameData")
            .ok_or_else(|| anyhow!("GameData record set not found"))?;

        let mut nodes = BTreeMap::new();
        let mut start = None;
        for record in rset.records() {
            let node = parse_node(&record)?;
            start.get_or_insert(node.id);
            if nodes.insert(node.id, node).is_some() {
                return Err(anyhow!("duplicate node id"));
            }
        }
        let start = start.ok_or_else(|| anyhow!("GameData has no records"))?;

        for node in nodes.values() {
            for target in &node.connects_to {
                if !nodes.contains_key(target) {
                    return Err(anyhow!("node {} connects_to unknown node {target}", node.id));
                }
            }
        }

        Ok(Game {
            nodes,
            start,
            discovered: vec![start],
            completed: BTreeSet::new(),
            inventory: BTreeMap::new(),
            active: None,
            time_used: 0.0,
            max_time,
            state: STATE_PLAYING.to_string(),
        })
    }

    pub fn node(&self, id: u32) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn start_id(&self) -> u32 {
        self.start
    }

    pub fn is_discovered(&self, id: u32) -> bool {
        self.discovered.contains(&id)
    }

    /// Discovered node ids in discovery order.
    pub fn discovered(&self) -> &[u32] {
        &self.discovered
    }

    pub fn inventory(&self) -> &BTreeMap<String, u32> {
        &self.inventory
    }

    pub fn item_count(&self, item: &str) -> u32 {
        self.inventory.get(item).copied().unwrap_or(0)
    }

    pub fn time_used(&self) -> f64 {
        self.time_used
    }

    pub fn max_time(&self) -> f64 {
        self.max_time
    }

    pub fn time_left(&self) -> f64 {
        (self.max_time - self.time_used).max(0.0)
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn is_over(&self) -> bool {
        self.state.ends_with("ending") || self.state == STATE_OUT_OF_TIME
    }

    /// The in-progress action, as (node id, fraction complete in 0..=1).
    pub fn active_action(&self) -> Option<(u32, f64)> {
        self.active.as_ref().map(|a| {
            let total = self.nodes[&a.id].time_to_complete;
            let done = if total > 0.0 { 1.0 - a.remaining / total } else { 1.0 };
            (a.id, done.clamp(0.0, 1.0))
        })
    }

    pub fn missing_requirements(&self, id: u32) -> Vec<String> {
        self.nodes
            .get(&id)
            .map(|n| {
                n.requirements
                    .iter()
                    .filter(|r| self.item_count(r) == 0)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn node_status(&self, id: u32) -> NodeStatus {
        if self.completed.contains(&id) {
            NodeStatus::Completed
        } else if self.active.as_ref().is_some_and(|a| a.id == id) {
            NodeStatus::InProgress
        } else if !self.missing_requirements(id).is_empty() {
            NodeStatus::Locked
        } else {
            NodeStatus::Available
        }
    }

    pub fn start_action(&mut self, id: u32) -> Result<(), ActionError> {
        if self.is_over() {
            return Err(ActionError::GameOver);
        }
        if self.active.is_some() {
            return Err(ActionError::Busy);
        }
        let node = self.nodes.get(&id).ok_or(ActionError::UnknownNode(id))?;
        if !self.is_discovered(id) {
            return Err(ActionError::NotDiscovered);
        }
        if self.completed.contains(&id) {
            return Err(ActionError::AlreadyCompleted);
        }
        let missing = self.missing_requirements(id);
        if !missing.is_empty() {
            return Err(ActionError::MissingItems(missing));
        }
        self.active = Some(ActiveAction {
            id,
            remaining: node.time_to_complete,
        });
        Ok(())
    }

    /// Advance time by `dt` seconds. Time is only consumed while an action is
    /// in progress, so thinking is free and acting costs the time budget.
    pub fn tick(&mut self, dt: f64) -> Vec<Event> {
        let mut events = Vec::new();
        if self.is_over() {
            return events;
        }
        let Some(active) = &mut self.active else {
            return events;
        };
        let consumed = dt.min(active.remaining);
        self.time_used += consumed;
        active.remaining -= consumed;

        if active.remaining <= 0.0 {
            let id = active.id;
            self.active = None;
            self.complete(id, &mut events);
        }
        if !self.is_over() && self.time_used >= self.max_time {
            self.state = STATE_OUT_OF_TIME.to_string();
            self.active = None;
            events.push(Event::StateChanged(self.state.clone()));
        }
        events
    }

    fn complete(&mut self, id: u32, events: &mut Vec<Event>) {
        self.completed.insert(id);
        let node = self.nodes[&id].clone();
        if let Some(log) = node.log {
            events.push(Event::Log { node_id: id, text: log });
        }
        for (item, quantity) in node.add_to_inventory {
            *self.inventory.entry(item.clone()).or_insert(0) += quantity;
            events.push(Event::ItemAdded { item, quantity });
        }
        for (item, quantity) in node.remove_from_inventory {
            let Some(held) = self.inventory.get_mut(&item) else {
                continue;
            };
            let removed = quantity.min(*held);
            *held -= removed;
            if *held == 0 {
                self.inventory.remove(&item);
            }
            events.push(Event::ItemRemoved { item, quantity: removed });
        }
        let newly: Vec<u32> = node
            .connects_to
            .iter()
            .copied()
            .filter(|c| !self.discovered.contains(c))
            .collect();
        if !newly.is_empty() {
            self.discovered.extend(&newly);
            events.push(Event::Discovered(newly));
        }
        if let Some(state) = node.state_change_to {
            self.state = state.clone();
            events.push(Event::StateChanged(state));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load() -> Game {
        let raw = std::fs::read_to_string("game.rec").unwrap();
        Game::from_rec_str(&raw).unwrap()
    }

    fn run_to_completion(game: &mut Game, id: u32) -> Vec<Event> {
        game.start_action(id).unwrap();
        let mut events = Vec::new();
        while game.active_action().is_some() {
            events.extend(game.tick(0.5));
        }
        events
    }

    #[test]
    fn only_start_is_discovered_initially() {
        let game = load();
        assert_eq!(game.discovered(), &[game.start_id()]);
    }

    #[test]
    fn requirements_gate_actions() {
        let mut game = load();
        for id in [0, 1, 2, 4] {
            run_to_completion(&mut game, id);
        }
        // Weld (13) needs the iron chain we don't have yet.
        run_to_completion(&mut game, 11);
        run_to_completion(&mut game, 12);
        assert_eq!(
            game.start_action(13),
            Err(ActionError::MissingItems(vec!["iron-chain".into()]))
        );
    }

    #[test]
    fn good_ending_is_reachable_within_time() {
        let mut game = load();
        for id in [0, 1, 2, 4, 5, 6, 9, 10, 11, 12, 13, 7, 8, 14, 16] {
            run_to_completion(&mut game, id);
        }
        assert_eq!(game.state(), "good-ending");
        assert!(game.is_over());
        assert!(game.time_used() <= game.max_time());
    }

    #[test]
    fn bad_ending_ends_the_game() {
        let mut game = load();
        for id in [0, 1, 14, 15] {
            run_to_completion(&mut game, id);
        }
        assert_eq!(game.state(), "bad-ending");
        assert_eq!(game.start_action(16), Err(ActionError::GameOver));
    }

    #[test]
    fn completed_actions_cannot_be_repeated() {
        let mut game = load();
        run_to_completion(&mut game, 0);
        assert_eq!(game.start_action(0), Err(ActionError::AlreadyCompleted));
    }

    #[test]
    fn quantities_add_and_remove() {
        let raw = "%rec: GameData\n\n\
            id: 0\ntitle: Pick berries\nadd_to_inventory: berry\nadd_quantity: 3\n\
            connects_to: 1\ntime_to_complete: 1\n\n\
            id: 1\ntitle: Eat berries\nrequirement: berry\n\
            remove_from_inventory: berry\nremove_quantity: 2\ntime_to_complete: 1\n";
        let mut game = Game::from_rec_str(raw).unwrap();
        run_to_completion(&mut game, 0);
        assert_eq!(game.item_count("berry"), 3);
        run_to_completion(&mut game, 1);
        assert_eq!(game.item_count("berry"), 1);
    }

    #[test]
    fn removal_saturates_at_zero_and_drops_the_entry() {
        let raw = "%rec: GameData\n\n\
            id: 0\ntitle: Give away\nadd_to_inventory: coin\n\
            remove_from_inventory: coin\nremove_quantity: 5\ntime_to_complete: 1\n";
        let mut game = Game::from_rec_str(raw).unwrap();
        run_to_completion(&mut game, 0);
        assert_eq!(game.item_count("coin"), 0);
        assert!(game.inventory().is_empty());
    }

    #[test]
    fn running_out_of_time_ends_the_game() {
        let raw = "%rec: GameMetadata\n\nmax_time: 2\n\n%rec: GameData\n\nid: 0\ntitle: Dawdle\ntime_to_complete: 5\n";
        let mut game = Game::from_rec_str(raw).unwrap();
        game.start_action(0).unwrap();
        game.tick(10.0);
        assert_eq!(game.state(), STATE_OUT_OF_TIME);
        assert!(game.is_over());
    }
}
