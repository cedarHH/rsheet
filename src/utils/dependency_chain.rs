use lazy_static::lazy_static;
use petgraph::graph::{DiGraph, NodeIndex, UnGraph};
use petgraph::algo::{toposort, tarjan_scc};
use petgraph::visit::{EdgeRef, Walker, IntoNodeReferences, NodeRef, Dfs};
use std::sync::RwLock;
use std::collections::{HashMap, HashSet};

// Global variables for static lifecycle
// Store all dependencies using a directed graph
lazy_static! {
    pub static ref DEPENDENCIES: RwLock<DiGraph<(u32, u32), ()>> = RwLock::new(DiGraph::new());
}

// Enum of errors in topological ordering
enum TopoError {
    NodeNotFound,
    CycleDetected(Vec<(u32, u32)>),
}

// Update the dependency of node B
pub fn update_incoming_edges(a_vec: Vec<(u32, u32)>, b: (u32, u32)) {
    let mut graph = DEPENDENCIES.write().unwrap();

    let node_b = find_or_add_node(&mut graph, b);

    let incoming_edges_to_remove: Vec<_> = graph.edges_directed(node_b, petgraph::Direction::Incoming)
        .map(|edge| edge.id())
        .collect();
    for edge_id in incoming_edges_to_remove {
        graph.remove_edge(edge_id);
    }

    for a in a_vec {
        let node_a = find_or_add_node(&mut graph, a);
        graph.update_edge(node_a, node_b, ());
    }
}

fn find_or_add_node(graph: &mut DiGraph<(u32, u32), ()>, node: (u32, u32)) -> NodeIndex {
    if let Some(index) = graph.node_indices().find(|&i| graph[i] == node) {
        index
    } else {
        graph.add_node(node)
    }
}

// Calculating dependency chains, or self-dependency
// If the node is not in the directed graph returns `TopoError<NodeNotFound>`
// If the node is in cycle,
//     returns all nodes `TopoError<CycleDetected(Vec<(u32,u32)>)>` in the strongly connected component in which the node is located
//     and nodes that depend on nodes in the strongly connected component.
// If the weakly connected component where the node is located is a directed acyclic graph,
//     return the topological ordering `Vec<(u32,u32)`
pub fn find_topology_sort_of_weakly_component(node: (u32, u32)) -> Result<Vec<(u32, u32)>, TopoError> {
    let graph = DEPENDENCIES.read().unwrap();

    // Convert directed graph to undirected graph
    // The weakly connected components of a directed graph are the same as
    // the connected components of the corresponding undirected graph
    let mut undirected_graph = UnGraph::<(u32, u32), ()>::new_undirected();
    let mut node_to_index = HashMap::new();

    for node_ref in graph.node_references() {
        let (a, b) = *node_ref.weight();
        node_to_index.insert((a, b), undirected_graph.add_node((a, b)));
    }

    for edge_ref in graph.edge_references() {
        let source = undirected_graph.node_weight(node_to_index[&graph[edge_ref.source()]]);
        let target = undirected_graph.node_weight(node_to_index[&graph[edge_ref.target()]]);
        if let (Some(source), Some(target)) = (source, target) {
            undirected_graph.update_edge(node_to_index[source], node_to_index[target], ());
        }
    }

    // If the node is not in the directed graph returns `TopoError<NodeNotFound>`
    let node_index = node_to_index.get(&node);
    if node_index.is_none() {
        return Err(TopoError::NodeNotFound);
    }
    let node_index = *node_index.unwrap();

    // Using DFS algorithm to calculate the connected components of an undirected graph
    let mut connected_component = HashSet::from([node_index]);
    let mut dfs = Dfs::new(&undirected_graph, node_index);
    while let Some(node_idx) = dfs.next(&undirected_graph) {
        connected_component.insert(node_idx);
    }

    // Find the maximal connected subgraph
    let subgraph_nodes: Vec<NodeIndex> = connected_component.iter().map(|&index| index).collect();
    let subgraph = graph.filter_map(|index, weight|
                                        if subgraph_nodes.contains(&index) { Some(*weight) }
                                        else { None },
                                    |index, weight| Some(*weight));

    // Perform topological sort
    match toposort(&subgraph, None) {
        //If the weakly connected component where the node is located is a directed acyclic graph,
        //     return the topological ordering `Vec<(u32,u32)`
        Ok(sorted_indices) => {
            let sorted_values = sorted_indices.iter()
                .map(|&index| *subgraph.node_weight(index).unwrap())
                .collect::<Vec<_>>();
            Ok(sorted_values)
        }
        // If the node is in cycle,
        //     returns all nodes `TopoError<CycleDetected(Vec<(u32,u32)>)>` in the strongly connected component in which the node is located
        //     and nodes that depend on nodes in the strongly connected component.
        Err(cycle) => {
            let cycle_node_id = cycle.node_id();
            // Compute strongly connected component on the graph
            let scc = tarjan_scc(&*graph);

            // Find the strongly connected component that contains the cycle node
            let cycle_component = scc.iter().find(|comp| comp.contains(&cycle_node_id)).unwrap();

            // Find the nodes that depend on the strongly connected components
            let mut visited_nodes = HashSet::new();
            for node_idx in cycle_component {
                let mut dfs = Dfs::new(&*graph, *node_idx);
                while let Some(nx) = dfs.next(&*graph) {
                    visited_nodes.insert(*graph.node_weight(nx).unwrap());
                }
            }

            Err(TopoError::CycleDetected(visited_nodes.into_iter().collect()))
        }
    }
}

// Test functions for algorithms such as
// construction of dependency graphs,
// compute the topological ordering of weakly connected components,
// and finding rings (strongly connected components)

// #[allow(dead_code)]
// pub fn __test() {
//
//     // test1:
//     // Graph:
//     // 1 -> 4 -> 5  6 -> 7
//     // ↓ ↗ ↓ ↗      ↗
//     // 2 -> 3       8
//     // Topological sort:
//     // 1->2->4->3->5
//
//     update_incoming_edges([(1, 1), (2, 2)].to_vec(), (4, 4));
//     update_incoming_edges([(1, 1)].to_vec(), (2, 2));
//     update_incoming_edges([(3, 3), (4, 4)].to_vec(), (5, 5));
//     update_incoming_edges([(2, 2), (4, 4)].to_vec(), (3, 3));
//     update_incoming_edges([(6, 6), (8, 8)].to_vec(), (7, 7));
//     let graph = DEPENDENCIES.read().unwrap();
//     println!("{:?}", graph);
//     match find_topology_sort_of_weakly_component((2, 2)) {
//         Ok(topo_sort) => println!("Topological sort: {:?}", topo_sort),
//         Err(TopoError::NodeNotFound) => println!("Error: Node not found"),
//         Err(TopoError::CycleDetected(cycle)) => println!("Detected cycle: {:?}", cycle),
//     };
//     drop(graph);
//
//     // test2:
//     // Graph:
//     // 1 -> 4 -> 5 -> 9  6 -> 7
//     // ↓ ↗ ↓ ↗           ↗
//     // 2 -> 3            8
//     // Topological sort:
//     // 1->2->4->3->5->9
//
//     update_incoming_edges([(5, 5)].to_vec(), (9, 9));
//     let graph = DEPENDENCIES.read().unwrap();
//     println!("{:?}", graph);
//     match find_topology_sort_of_weakly_component((2, 2)) {
//         Ok(topo_sort) => println!("Topological sort: {:?}", topo_sort),
//         Err(TopoError::NodeNotFound) => println!("Error: Node not found"),
//         Err(TopoError::CycleDetected(cycle)) => println!("Detected cycle: {:?}", cycle),
//     }
//     drop(graph);
//
//     //test3:
//     // Graph:
//     // 1 -> 4 -> 5 -> 9  6 -> 7
//     // ↓ ↗   ↗           ↗
//     // 2    3            8
//     // Topological sort:
//     // 3->1->2->4->5->9
//
//     update_incoming_edges([].to_vec(), (3, 3));
//     let graph = DEPENDENCIES.read().unwrap();
//     println!("{:?}", graph);
//     match find_topology_sort_of_weakly_component((2, 2)) {
//         Ok(topo_sort) => println!("Topological sort: {:?}", topo_sort),
//         Err(TopoError::NodeNotFound) => println!("Error: Node not found"),
//         Err(TopoError::CycleDetected(cycle)) => println!("Detected cycle: {:?}", cycle),
//     }
//     drop(graph);
//
//     // test4:
//     // Graph:
//     // 3 -> 5 -> 9
//     //   ↙ ↑
//     // 1 -> 4    6 -> 7
//     // ↓ ↗        ↗
//     // 2         8
//     // Detected cycle
//     // Return the nodes in the strongly connected component
//     // and the nodes that depend on the nodes in the strongly connected component
//     // [2,1,5,4,9]
//     // Strongly connected component:
//     // [2,1,5,4]
//     // Nodes depend on strongly connected components
//     // [9]
//
//     update_incoming_edges([(5, 5)].to_vec(), (1, 1));
//     let graph = DEPENDENCIES.read().unwrap();
//     println!("{:?}", graph);
//     match find_topology_sort_of_weakly_component((2, 2)) {
//         Ok(topo_sort) => println!("Topological sort: {:?}", topo_sort),
//         Err(TopoError::NodeNotFound) => println!("Error: Node not found"),
//         Err(TopoError::CycleDetected(cycle)) => println!("Detected cycle: {:?}", cycle),
//     }
//     drop(graph);
//
//     // test5:
//     // Graph:
//     // 3 -> 5 -> 9
//     //   ↙
//     // 1 -> 4    6 -> 7
//     // ↓ ↗        ↗
//     // 2         8
//     // Topological sort:
//     // 3->5->9->1->2->4
//
//     update_incoming_edges([(3, 3)].to_vec(), (5, 5));
//     let graph = DEPENDENCIES.read().unwrap();
//     println!("{:?}", graph);
//     match find_topology_sort_of_weakly_component((2, 2)) {
//         Ok(topo_sort) => println!("Topological sort: {:?}", topo_sort),
//         Err(TopoError::NodeNotFound) => println!("Error: Node not found"),
//         Err(TopoError::CycleDetected(cycle)) => println!("Detected cycle: {:?}", cycle),
//     }
// }