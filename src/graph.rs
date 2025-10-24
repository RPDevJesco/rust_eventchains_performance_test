use std::cmp::Ordering;

/// Node in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Edge with weight
#[derive(Debug, Clone)]
pub struct Edge {
    pub to: NodeId,
    pub weight: u32,
}

/// Graph representation
#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: usize,
    pub adjacency_list: Vec<Vec<Edge>>,
}

impl Graph {
    pub fn new(nodes: usize) -> Self {
        Self {
            nodes,
            adjacency_list: vec![Vec::new(); nodes],
        }
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: u32) {
        self.adjacency_list[from.0].push(Edge { to, weight });
    }

    pub fn add_bidirectional_edge(&mut self, from: NodeId, to: NodeId, weight: u32) {
        self.add_edge(from, to, weight);
        self.add_edge(to, from, weight);
    }

    /// Generate a random connected graph
    pub fn random_connected(nodes: usize, edges: usize, max_weight: u32) -> Self {
        use hashbrown::HashSet;

        let mut graph = Graph::new(nodes);
        let mut rng = SimpleRng::new(12345);
        let mut edge_set = HashSet::new();

        // Ensure connectivity by creating a spanning tree
        for i in 1..nodes {
            let parent = rng.next_usize() % i;
            let weight = (rng.next_usize() % max_weight as usize) as u32 + 1;
            graph.add_bidirectional_edge(NodeId(parent), NodeId(i), weight);
            edge_set.insert((parent.min(i), parent.max(i)));
        }

        // Add remaining random edges
        let mut added = nodes - 1;
        let mut attempts = 0;
        while added < edges && attempts < edges * 10 {
            let from = rng.next_usize() % nodes;
            let to = rng.next_usize() % nodes;

            if from != to {
                let edge_key = (from.min(to), from.max(to));
                if edge_set.insert(edge_key) {
                    let weight = (rng.next_usize() % max_weight as usize) as u32 + 1;
                    graph.add_bidirectional_edge(NodeId(from), NodeId(to), weight);
                    added += 1;
                }
            }
            attempts += 1;
        }

        graph
    }
}

/// Simple RNG for reproducible results
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_usize(&mut self) -> usize {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as usize
    }
}

/// State for Dijkstra's algorithm
#[derive(Debug, Clone)]
pub struct DijkstraState {
    pub distances: Vec<u32>,
    pub predecessors: Vec<Option<NodeId>>,
    pub visited: Vec<bool>,
}

impl DijkstraState {
    pub fn new(nodes: usize, source: NodeId) -> Self {
        let mut distances = vec![u32::MAX; nodes];
        distances[source.0] = 0;

        Self {
            distances,
            predecessors: vec![None; nodes],
            visited: vec![false; nodes],
        }
    }
}

/// Priority queue node for Dijkstra's algorithm
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct QueueNode {
    pub node: NodeId,
    pub distance: u32,
}

impl Ord for QueueNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other
            .distance
            .cmp(&self.distance)
            .then_with(|| self.node.0.cmp(&other.node.0))
    }
}

impl PartialOrd for QueueNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Result of shortest path computation
#[derive(Debug, Clone)]
pub struct ShortestPathResult {
    pub source: NodeId,
    pub target: NodeId,
    pub distance: Option<u32>,
    pub path: Vec<NodeId>,
}

impl ShortestPathResult {
    pub fn reconstruct_path(state: &DijkstraState, source: &NodeId, target: &NodeId) -> Self {
        let distance = if state.distances[target.0] == u32::MAX {
            None
        } else {
            Some(state.distances[target.0])
        };

        let mut path: Vec<NodeId> = Vec::new();

        if distance.is_some() {
            let mut current = *target;
            while current != *source {
                path.push(current);
                if let Some(pred) = state.predecessors[current.0] {
                    current = pred;
                } else {
                    break;
                }
            }
            path.push(*source);
            path.reverse();
        }

        Self {
            source: *source,
            target: *target,
            distance,
            path,
        }
    }
}
