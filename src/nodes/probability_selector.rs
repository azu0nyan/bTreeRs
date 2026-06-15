//! The [`ProbabilitySelector`] composite.

use crate::caps::HasRng;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Picks one child at random — weighted by per-child probabilities — and runs
/// it to completion. Mirrors `BTProbabilitySelector`.
///
/// On entry a single child is chosen by a weighted draw from the context's
/// [`HasRng`]; that child is then ticked (across as many ticks as it needs) and
/// its result returned. Once it finishes the choice is cleared so the next
/// entry re-draws. If the weights sum to zero the node fails.
pub struct ProbabilitySelector<D> {
    weights: Vec<f64>,
    children: Vec<BoxNode<D>>,
    chosen: Option<usize>,
}

impl<D> ProbabilitySelector<D> {
    /// Build from `(weight, child)` pairs. Weights need not sum to 1; they are
    /// normalized by their total at draw time. Negative weights are clamped to
    /// zero.
    pub fn new(weighted: Vec<(f64, BoxNode<D>)>) -> Self {
        let mut weights = Vec::with_capacity(weighted.len());
        let mut children = Vec::with_capacity(weighted.len());
        for (w, child) in weighted {
            weights.push(w.max(0.0));
            children.push(child);
        }
        Self {
            weights,
            children,
            chosen: None,
        }
    }
}

impl<D: HasRng> BehaviorNode<D> for ProbabilitySelector<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        if self.chosen.is_none() {
            let total: f64 = self.weights.iter().sum();
            if total <= 0.0 {
                record(dbg, Status::Failure, || {
                    "ProbabilitySelector (no weight)".to_string()
                });
                return Status::Failure;
            }
            let mut roll = data.next_f64() * total;
            let mut pick = self.weights.len() - 1;
            for (i, w) in self.weights.iter().enumerate() {
                if roll < *w {
                    pick = i;
                    break;
                }
                roll -= *w;
            }
            self.chosen = Some(pick);
        }
        let idx = self.chosen.unwrap();
        let status = tick_child(&mut *self.children[idx], data, &mut dbg);
        if status.is_done() {
            self.chosen = None;
        }
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        if let Some(idx) = self.chosen {
            self.children[idx].halt();
        }
        self.chosen = None;
    }

    fn node_info(&self) -> String {
        match self.chosen {
            Some(i) => format!("ProbabilitySelector -> {i}"),
            None => "ProbabilitySelector".to_string(),
        }
    }
}
