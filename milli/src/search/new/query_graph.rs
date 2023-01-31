use super::query_term::{LocatedQueryTerm, QueryTerm, WordDerivations};
use std::{collections::HashSet, fmt::Debug};

#[derive(Clone)]
pub enum QueryNode {
    Term(LocatedQueryTerm),
    Deleted,
    Start,
    End,
}

#[derive(Debug, Clone)]
pub struct Edges {
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

#[derive(Debug)]
pub struct QueryGraph {
    pub root_node: usize,
    pub end_node: usize,
    pub nodes: Vec<QueryNode>,
    pub edges: Vec<Edges>,
}

fn _assert_sizes() {
    let _: [u8; 56] = [0; std::mem::size_of::<QueryNode>()];
    let _: [u8; 96] = [0; std::mem::size_of::<Edges>()];
}

impl Default for QueryGraph {
    /// Create a new QueryGraph with two disconnected nodes: the root and end nodes.
    fn default() -> Self {
        let nodes = vec![QueryNode::Start, QueryNode::End];
        let edges = vec![
            Edges { incoming: HashSet::new(), outgoing: HashSet::new() },
            Edges { incoming: HashSet::new(), outgoing: HashSet::new() },
        ];

        Self { root_node: 0, end_node: 1, nodes, edges }
    }
}

impl QueryGraph {
    fn connect_to_node(&mut self, from_nodes: &[usize], end_node: usize) {
        for &from_node in from_nodes {
            self.edges[from_node].outgoing.insert(end_node);
            self.edges[end_node].incoming.insert(from_node);
        }
    }
    fn add_node(&mut self, from_nodes: &[usize], node: QueryNode) -> usize {
        let new_node_idx = self.nodes.len();
        self.nodes.push(node);
        self.edges.push(Edges {
            incoming: from_nodes.iter().copied().collect(),
            outgoing: HashSet::new(),
        });
        for from_node in from_nodes {
            self.edges[*from_node].outgoing.insert(new_node_idx);
        }
        new_node_idx
    }
}

impl QueryGraph {
    // TODO: return the list of all matching words here as well

    pub fn from_query(query: Vec<LocatedQueryTerm>, word_set: fst::Set<Vec<u8>>) -> QueryGraph {
        let mut graph = QueryGraph::default();

        let (mut prev2, mut prev1, mut prev0): (Vec<usize>, Vec<usize>, Vec<usize>) =
            (vec![], vec![], vec![graph.root_node]);

        // TODO: add all the word derivations found in the fst
        // and add split words / support phrases

        for length in 1..=query.len() {
            let query = &query[..length];

            let term0 = query.last().unwrap();

            let mut new_nodes = vec![];
            let ngram1_idx = graph.add_node(&prev0, QueryNode::Term(term0.clone()));
            new_nodes.push(ngram1_idx);

            if !prev1.is_empty() {
                if let Some((ngram2_str, ngram2_pos)) =
                    LocatedQueryTerm::ngram2(&query[length - 2], &query[length - 1])
                {
                    if word_set.contains(ngram2_str.as_bytes()) {
                        println!("word set contains {ngram2_str}? yes");
                        let ngram2 = LocatedQueryTerm {
                            value: QueryTerm::Word {
                                original: ngram2_str,
                                derivations: WordDerivations::FromList(vec![]),
                            },
                            positions: ngram2_pos,
                        };
                        let ngram2_idx = graph.add_node(&prev1, QueryNode::Term(ngram2));
                        new_nodes.push(ngram2_idx);
                    } else {
                        println!("word set contains {ngram2_str}? no");
                    }
                }
            }
            if !prev2.is_empty() {
                if let Some((ngram3_str, ngram3_pos)) = LocatedQueryTerm::ngram3(
                    &query[length - 3],
                    &query[length - 2],
                    &query[length - 1],
                ) {
                    if word_set.contains(ngram3_str.as_bytes()) {
                        let ngram3 = LocatedQueryTerm {
                            value: QueryTerm::Word {
                                original: ngram3_str,
                                derivations: WordDerivations::FromList(vec![]),
                            },
                            positions: ngram3_pos,
                        };
                        let ngram3_idx = graph.add_node(&prev2, QueryNode::Term(ngram3));
                        new_nodes.push(ngram3_idx);
                    }
                }
            }
            (prev0, prev1, prev2) = (new_nodes, prev0, prev1);
        }
        graph.connect_to_node(&prev0, graph.end_node);

        graph
    }
    pub fn remove_nodes(&mut self, nodes: &[usize]) {
        for &node in nodes {
            self.nodes[node] = QueryNode::Deleted;
            let edges = self.edges[node].clone();
            for &pred in edges.incoming.iter() {
                self.edges[pred].outgoing.remove(&node);
            }
            for succ in edges.outgoing {
                self.edges[succ].incoming.remove(&node);
            }
            self.edges[node] = Edges { incoming: HashSet::new(), outgoing: HashSet::new() };
        }
    }
    pub fn remove_nodes_keep_edges(&mut self, nodes: &[usize]) {
        for &node in nodes {
            self.nodes[node] = QueryNode::Deleted;
            let edges = self.edges[node].clone();
            for &pred in edges.incoming.iter() {
                self.edges[pred].outgoing.remove(&node);
                self.edges[pred].outgoing.extend(edges.outgoing.iter());
            }
            for succ in edges.outgoing {
                self.edges[succ].incoming.remove(&node);
                self.edges[succ].incoming.extend(edges.incoming.iter());
            }
            self.edges[node] = Edges { incoming: HashSet::new(), outgoing: HashSet::new() };
        }
    }
    pub fn remove_words_at_position(&mut self, position: i8) {
        let mut nodes_to_remove_keeping_edges = vec![];
        let mut nodes_to_remove = vec![];
        for (node_idx, node) in self.nodes.iter().enumerate() {
            let QueryNode::Term(LocatedQueryTerm { value: _, positions }) = node else { continue };
            if positions == &(position..=position) {
                nodes_to_remove_keeping_edges.push(node_idx)
            } else if positions.contains(&position) {
                nodes_to_remove.push(node_idx)
            }
        }

        self.remove_nodes(&nodes_to_remove);
        self.remove_nodes_keep_edges(&nodes_to_remove_keeping_edges);

        self.simplify();
    }

    fn simplify(&mut self) {
        loop {
            let mut nodes_to_remove = vec![];
            for (node_idx, node) in self.nodes.iter().enumerate() {
                if (!matches!(node, QueryNode::End | QueryNode::Deleted)
                    && self.edges[node_idx].outgoing.is_empty())
                    || (!matches!(node, QueryNode::Start | QueryNode::Deleted)
                        && self.edges[node_idx].incoming.is_empty())
                {
                    nodes_to_remove.push(node_idx);
                }
            }
            if nodes_to_remove.is_empty() {
                break;
            } else {
                self.remove_nodes(&nodes_to_remove);
            }
        }
    }
}
impl Debug for QueryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryNode::Term(LocatedQueryTerm { value, positions: _ }) => match value {
                QueryTerm::Word { original: w, derivations } => {
                    write!(f, "\"{w} ")?;
                    match derivations {
                        WordDerivations::FromList(derived_words) => {
                            write!(f, "({}D)", derived_words.len())?;
                        }
                        WordDerivations::FromPrefixDB => {
                            write!(f, "(P)")?;
                        }
                    }
                    write!(f, "\"")?;
                    Ok(())
                }
                QueryTerm::Phrase(ws) => {
                    let joined =
                        ws.iter().filter_map(|x| x.clone()).collect::<Vec<String>>().join(" ");
                    let in_quotes = format!("\"{joined}\"");
                    let escaped = in_quotes.escape_default().collect::<String>();
                    write!(f, "\"{escaped}\"")
                }
            },
            QueryNode::Start => write!(f, "\"START\""),
            QueryNode::End => write!(f, "\"END\""),
            QueryNode::Deleted => write!(f, "\"_deleted_\""),
        }
    }
}

/*
TODO:

1. Find the minimum number of words to check to resolve the 10 query trees at once.
    (e.g. just 0 | 01 | 012 )
2. Simplify the query tree after removal of a node ✅
3. Create the proximity graph ✅
4. Assign different proximities for the ngrams ✅
5. Walk the proximity graph, finding all the potential paths of weight N from START to END ✅
(without checking the bitmaps)

*/
#[cfg(test)]
mod tests {
    use super::{LocatedQueryTerm, QueryGraph, QueryNode};
    use crate::{index::tests::TempIndex, search::new::query_term::word_derivations_max_typo_1};
    use charabia::Tokenize;

    impl QueryGraph {
        pub fn graphviz(&self) -> String {
            let mut desc = String::new();
            desc.push_str("digraph G {\nrankdir = LR;\n");

            for node in 0..self.nodes.len() {
                if matches!(self.nodes[node], QueryNode::Deleted) {
                    continue;
                }
                desc.push_str(&format!("{node} [label = {:?}]", &self.nodes[node],));
                if node == self.root_node {
                    desc.push_str("[color = blue]");
                } else if node == self.end_node {
                    desc.push_str("[color = red]");
                }
                desc.push_str(";\n");

                for edge in self.edges[node].outgoing.iter() {
                    desc.push_str(&format!("{node} -> {edge};\n"));
                }
                // for edge in self.edges[node].incoming.iter() {
                //     desc.push_str(&format!("{node} -> {edge} [color = grey];\n"));
                // }
            }

            desc.push('}');
            desc
        }
    }

    #[test]
    fn build_graph() {
        let index = TempIndex::new();
        let fst = fst::Set::from_iter(["01", "23", "234", "56"]).unwrap();

        let mut graph = QueryGraph::from_query(
            LocatedQueryTerm::from_query("0 1 2 3 4 5 6 7".tokenize(), None, |word, is_prefix| {
                word_derivations_max_typo_1(&index, word, is_prefix, &fst)
            })
            .unwrap(),
            fst,
        );
        // println!("{}", graph.graphviz());

        let positions_to_remove = vec![3, 6, 0, 4];
        for p in positions_to_remove {
            graph.remove_words_at_position(p);
            println!("{}", graph.graphviz());
        }

        // let proximities = |w1: &str, w2: &str| -> Vec<i8> {
        //     if matches!((w1, w2), ("56", "7")) {
        //         vec![]
        //     } else {
        //         vec![1, 2]
        //     }
        // };

        // let prox_graph = ProximityGraph::from_query_graph(graph, proximities);

        // println!("{}", prox_graph.graphviz());
    }
}

// fn remove_element_from_vector(v: &mut Vec<usize>, el: usize) {
//     let position = v.iter().position(|&x| x == el).unwrap();
//     v.swap_remove(position);
// }
