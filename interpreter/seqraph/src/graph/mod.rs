use petgraph::{
    graph::{
        DiGraph,
        EdgeIndex,
        NodeIndex,
    },
    visit::{
        EdgeRef
    },
    dot::{
        Dot,
    },
};
use std::{
    fmt::{
        Debug,
    },
    path::{
        PathBuf,
    },
};
use node::{
    NodeData,
    NodeWeight,
};
use edge::{
    EdgeData,
};
use std::ops::{
    Deref,
    DerefMut,
};

pub mod edge;
pub mod node;

#[derive(Debug)]
pub struct Graph<N, E>
    where N: NodeData,
          E: EdgeData,
{
    graph: DiGraph<NodeWeight<N>, E>,
}
impl<'a, N, E> Graph<N, E>
    where N: NodeData,
          E: EdgeData,
{
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
        }
    }
    /// Nodes

    /// Return a NodeIndex for a node with NodeData
    pub fn add_node(&mut self, element: N) -> NodeIndex {
        if let Some(i) = self.find_node_index(&element) {
            i
        } else {
            self.graph.add_node(
                NodeWeight::new(element)
            )
        }
    }
    /// Get NodeWeight for NodeIndex
    pub fn get_node_weight(&self, i: NodeIndex) -> Option<NodeWeight<N>> {
        self.graph
            .node_weight(i)
            .map(Clone::clone)
    }
    /// Find NodeIndex for NodeData, if any
    pub fn find_node_index(&self, element: &N) -> Option<NodeIndex> {
        self.graph
            .node_indices()
            .find(|i| self.graph[*i].data == *element)
            .map(|i| i.clone())
    }
    /// Return NodeWeight for NodeData, if any
    pub fn find_node_weight(&self, element: &N) -> Option<NodeWeight<N>> {
        let i = self.find_node_index(element)?;
        self.graph
            .node_weight(i)
            .map(Clone::clone)
    }
    /// Return any NodeIndices for NodeDatas
    pub fn find_node_indices(&'a self, es: impl Iterator<Item=&'a N> + 'a) -> Option<Vec<NodeIndex>> {
        es.map(|e| self.find_node_index(e)).collect()
    }
    /// Return any NodeWeights for NodeDatas
    pub fn find_node_weights(&'a self, es: impl Iterator<Item=&'a N> + 'a) -> Option<Vec<NodeWeight<N>>> {
        es.map(|e| self.find_node_weight(e)).collect()
    }
    /// True if NodeData has node in graph
    pub fn has_node(&self, element: &N) -> bool {
        self.find_node_index(element).is_some()
    }
    /// Map NodeIndices to weights
    pub fn map_to_node_weights(
        &'a self,
        is: impl Iterator<Item=&'a NodeIndex> + 'a
        ) -> impl Iterator<Item=NodeWeight<N>> + 'a {
        is.filter_map(move |i| self.get_node_weight(i.clone()))
    }
    /// Get all NodeIndices in the graph
    pub fn all_node_indices(&self) -> Vec<NodeIndex> {
        self.node_indices().collect()
    }
    /// Get all NodeWeights in the graph
    pub fn all_node_weights(&self) -> Vec<NodeWeight<N>> {
        self.raw_nodes().iter().map(|n| n.weight.clone()).collect()
    }

    /// Edges

    /// Return an EdgeIndex for an edge with weight between NodeIndices
    pub fn add_edge(&mut self, li: NodeIndex, ri: NodeIndex, w: E) -> EdgeIndex {
        let i = self.find_edge_index(li, ri, &w);
        let i = if let Some(i) = i {
            i
        } else {
            self.graph.add_edge(li, ri, w)
        };
        i
    }
    /// Add a new edge with weight between NodeDatas
    pub fn add_node_edge(&mut self, l: N, r: N, w: E) -> EdgeIndex {
        let li = self.add_node(l);
        let ri = self.add_node(r);
        self.graph.add_edge(li, ri, w)
    }
    /// Return weight of edge with index, if any
    pub fn get_edge_weight(&self, i: EdgeIndex) -> Option<E> {
        self.graph
            .edge_weight(i)
            .map(Clone::clone)
    }
    /// Map EdgeIndices to weights
    pub fn map_to_edge_weights(
        &'a self,
        is: impl Iterator<Item=&'a EdgeIndex> + 'a
        ) -> impl Iterator<Item=E> + 'a {
        is.filter_map(move |i| self.get_edge_weight(i.clone()))
    }
    /// Get all edge weights between NodeIndices
    pub fn get_edges(&self, li: NodeIndex, ri: NodeIndex) -> Vec<E> {
        self.map_to_edge_weights(self.find_edge_indices(li, ri).iter()).collect()
    }
    /// Get all edge weights between NodeDatas
    pub fn get_node_edges(&self, l: &N, r: &N) -> Vec<E> {
        self.map_to_edge_weights(self.find_node_edge_indices(l, r).iter()).collect()
    }
    /// Find edge index of edge with weight between NodeIndices
    pub fn find_edge_index(&self, li: NodeIndex, ri: NodeIndex, w: &E) -> Option<EdgeIndex> {
        self.graph
            .edges_connecting(li, ri)
            .find(|e| *e.weight() == *w)
            .map(|e| e.id())
    }
    /// Find edge index of edge with weight between NodeDatas
    pub fn find_node_edge_index(&self, l: &N, r: &N, w: &E) -> Option<EdgeIndex> {
        let li = self.find_node_index(l)?;
        let ri = self.find_node_index(r)?;
        self.find_edge_index(li, ri, w)
    }
    /// Find edge indices between NodeIndices
    pub fn find_edge_indices(&self, li: NodeIndex, ri: NodeIndex) -> Vec<EdgeIndex> {
        self.graph
            .edges_connecting(li, ri)
            .map(|e| e.id())
            .collect()
    }
    /// Find edge indices between NodeDatas
    pub fn find_node_edge_indices(&self, l: &N, r: &N) -> Vec<EdgeIndex> {
        let li = self.find_node_index(l);
        let ri = self.find_node_index(r);
        if let (Some(li), Some(ri)) = (li, ri) {
            self.graph
                .edges_connecting(li, ri)
                .map(|e| e.id())
                .collect()
        } else {
            Vec::new()
        }
    }
    /// Get all EdgeIndices in the graph
    pub fn all_edge_indices(&self) -> Vec<EdgeIndex> {
        self.edge_indices().collect()
    }
    /// True if edge with weight between NodeDatas
    pub fn has_node_edge(&self, l: &N, r: &N, w: &E) -> bool {
        self.find_node_edge_index(l, r, w).is_some()
    }
    /// True if edge with weight between NodeIndices
    pub fn has_edge(&self, li: NodeIndex, ri: NodeIndex, w: &E) -> bool {
        self.find_edge_index(li, ri, w).is_some()
    }
    /// Get source of edge
    pub fn edge_source(&self, i: EdgeIndex) -> Option<NodeIndex> {
        self.edge_endpoints(i).map(|t| t.0)
    }
    /// Get sources of edges
    pub fn edge_sources(&'a self, is: impl Iterator<Item=EdgeIndex> + 'a) -> impl Iterator<Item=NodeIndex> + 'a {
        is.filter_map(move |i| self.edge_source(i))
    }
    /// Get target of edge
    pub fn edge_target(&self, i: EdgeIndex) -> Option<NodeIndex> {
        self.edge_endpoints(i).map(|t| t.1)
    }
    /// Get targets of edges
    pub fn edge_targets(&'a self, is: impl Iterator<Item=EdgeIndex> + 'a) -> impl Iterator<Item=NodeIndex> + 'a {
        is.filter_map(move |i| self.edge_target(i))
    }
    /// Get weights of edges
    pub fn edge_weights(&'a self, is: impl Iterator<Item=EdgeIndex> + 'a) -> impl Iterator<Item=E> + 'a {
        is.filter_map(move |i| self.get_edge_weight(i))
    }
    /// Write graph to dot file
    pub fn write_to_file<S: Into<PathBuf>>(&self, name: S) -> std::io::Result<()> {
        let mut path: PathBuf = name.into();
        path.set_extension("dot");
        //path.canonicalize()?;
        path.parent().map(|p|
            std::fs::create_dir_all(p.clone())
        );
        std::fs::write(path, format!("{:?}", Dot::new(&self.graph)))
    }
}
impl<N: NodeData> Graph<N, usize> {
    /// Group NodeIndices by distances
    pub fn group_indices_by_distance(es: impl Iterator<Item=(usize, NodeIndex)>) -> Vec<Vec<NodeIndex>> {
        let es = es.collect::<Vec<_>>();
        let max = es.iter().map(|(d, _)| d.clone()).max().unwrap_or(0);
        let mut r = Vec::new();
        for i in 1..=max {
            r.push(
                es.iter().filter(|(d, _)| *d == i)
                    .map(|(_, n)| n.clone())
                    .collect()
            )
        }
        r
    }
    /// Group NodeWeights by distances
    pub fn group_weights_by_distance(&self, es: impl Iterator<Item=(usize, NodeIndex)>) -> Vec<Vec<NodeWeight<N>>> {
        Self::group_indices_by_distance(es)
            .iter()
            .map(|is|
                 self.map_to_node_weights(is.iter()).collect()
            )
            .collect()
    }
}
impl<N: NodeData, E: EdgeData> Deref for Graph<N, E> {
    type Target = DiGraph<NodeWeight<N>, E>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl<N: NodeData, E: EdgeData> DerefMut for Graph<N, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Mutex,
        MutexGuard,
    };
    lazy_static!{
        static ref ELEMS: Vec<char> = {
            Vec::from(['a', 'b', 'c'])
        };
        static ref EDGES: Vec<(char, char, usize)> = {
            Vec::from([
                ('a', 'b', 1),
                ('b', 'c', 2),
                ('c', 'a', 3)
            ])
        };
        static ref G: Mutex<Graph<char, usize>> = Mutex::new(Graph::new());
        static ref NODE_INDICES: Vec<NodeIndex> = {
            let mut g = G.lock().unwrap();
            ELEMS.iter()
                .map(|e| g.add_node(e.clone()))
                .collect()
        };
        static ref EDGE_INDICES: Vec<EdgeIndex> = {
            let mut g = G.lock().unwrap();
            EDGES.iter()
                .map(|(l, r, w)|
                     g.add_node_edge(l.clone(), r.clone(), w.clone())
                )
                .collect()
        };
    }
    fn init() -> MutexGuard<'static, Graph<char, usize>> {
        format!("{:?}", *NODE_INDICES);
        format!("{:?}", *EDGE_INDICES);
        G.lock().unwrap()
    }
    #[test]
    fn has_node() {
        let g = init();
        for e in ELEMS.iter() {
            assert!(g.has_node(&e));
        }
    }
    #[test]
    fn has_node_edge() {
        let g = init();
        for (l, r, w) in EDGES.iter() {
            assert!(g.has_node_edge(l, r, w));
        }
    }
    #[test]
    fn write_to_file() {
        let g = init();
        g.write_to_file("test_graph").unwrap();
    }
}
