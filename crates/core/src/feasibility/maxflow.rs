//! In-house Dinic maximum bipartite flow implementation.
//!
//! Used by the feasibility checker to verify that panel capacity requirements
//! can be simultaneously satisfied across all panels of an exam under rule H4.

/// A directed edge in the residual flow network.
#[derive(Debug, Clone)]
pub struct Edge {
    pub to: usize,
    pub rev: usize,
    pub cap: i32,
    pub flow: i32,
}

/// Network graph for computing maximum flow using Dinic's algorithm.
pub struct DinicGraph {
    adj: Vec<Vec<Edge>>,
    level: Vec<i32>,
    ptr: Vec<usize>,
}

impl DinicGraph {
    /// Creates a new network graph with `n` nodes (0..n-1).
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            adj: vec![Vec::new(); n],
            level: vec![0; n],
            ptr: vec![0; n],
        }
    }

    /// Adds a directed edge with capacity `cap` from `from` to `to`.
    pub fn add_edge(&mut self, from: usize, to: usize, cap: i32) {
        let rev_from = self.adj[to].len();
        let rev_to = self.adj[from].len();
        self.adj[from].push(Edge {
            to,
            rev: rev_from,
            cap,
            flow: 0,
        });
        self.adj[to].push(Edge {
            to: from,
            rev: rev_to,
            cap: 0,
            flow: 0,
        });
    }

    /// Computes the maximum flow from `source` to `sink`.
    pub fn max_flow(&mut self, source: usize, sink: usize) -> i32 {
        let mut total_flow = 0;
        while self.bfs(source, sink) {
            self.ptr.fill(0);
            while let Some(pushed) = self.dfs(source, sink, i32::MAX) {
                if pushed == 0 {
                    break;
                }
                total_flow += pushed;
            }
        }
        total_flow
    }

    /// Returns the net forward flow along edges from `from` to `to`.
    #[must_use]
    pub fn get_flow(&self, from: usize, to: usize) -> i32 {
        for e in &self.adj[from] {
            if e.to == to {
                return e.flow;
            }
        }
        0
    }

    fn bfs(&mut self, source: usize, sink: usize) -> bool {
        self.level.fill(-1);
        self.level[source] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(source);

        while let Some(u) = queue.pop_front() {
            for edge in &self.adj[u] {
                if edge.cap - edge.flow > 0 && self.level[edge.to] < 0 {
                    self.level[edge.to] = self.level[u] + 1;
                    queue.push_back(edge.to);
                }
            }
        }

        self.level[sink] >= 0
    }

    fn dfs(&mut self, u: usize, sink: usize, pushed: i32) -> Option<i32> {
        if pushed == 0 || u == sink {
            return Some(pushed);
        }

        for cid in self.ptr[u]..self.adj[u].len() {
            self.ptr[u] = cid;
            let edge = self.adj[u][cid].clone();
            let tr = edge.to;

            if self.level[u] + 1 != self.level[tr] || edge.cap - edge.flow == 0 {
                continue;
            }

            let pushable = pushed.min(edge.cap - edge.flow);
            if let Some(tr_pushed) = self.dfs(tr, sink, pushable) {
                if tr_pushed > 0 {
                    self.adj[u][cid].flow += tr_pushed;
                    let rev = edge.rev;
                    self.adj[tr][rev].flow -= tr_pushed;
                    return Some(tr_pushed);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dinic_basic_flow() {
        let mut graph = DinicGraph::new(4);
        // 0: source, 3: sink
        graph.add_edge(0, 1, 10);
        graph.add_edge(0, 2, 10);
        graph.add_edge(1, 2, 2);
        graph.add_edge(1, 3, 4);
        graph.add_edge(2, 3, 8);

        let flow = graph.max_flow(0, 3);
        assert_eq!(flow, 12);
    }

    #[test]
    fn test_h4_exam_bottleneck_scenario() {
        // Scenario: Exam has 2 panels (Grade 10, Grade 11), needing 3 slots each (6 total).
        // 5 teachers are available and qualified for BOTH grades.
        // Each grade individually has 5 qualified teachers (>= 3).
        // Under H4 (each teacher can be used at most once), only 5 slots can be filled,
        // so total flow must be 5 < 6.
        let mut graph = DinicGraph::new(9);
        let source = 0;
        let sink = 8;
        let g10 = 6;
        let g11 = 7;

        // Source -> 5 teachers with cap 1
        for t in 1..=5 {
            graph.add_edge(source, t, 1);
            // Each teacher can teach both G10 and G11
            graph.add_edge(t, g10, 1);
            graph.add_edge(t, g11, 1);
        }

        // Each panel needs 3 slots
        graph.add_edge(g10, sink, 3);
        graph.add_edge(g11, sink, 3);

        let total_flow = graph.max_flow(source, sink);
        assert_eq!(total_flow, 5, "expected bottleneck flow of 5");
        let flow_g10 = graph.get_flow(g10, sink);
        let flow_g11 = graph.get_flow(g11, sink);
        assert_eq!(flow_g10 + flow_g11, 5);
        assert!(flow_g10 < 3 || flow_g11 < 3);
    }
}
