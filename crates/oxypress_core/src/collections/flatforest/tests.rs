use std::char::from_u32;

use rand::RngExt;

use super::*;

#[derive(Copy, Clone, Debug)]
struct UnitTestTag;
type FlatForest<T> = super::FlatForest<UnitTestTag, T>;

impl<T> FlatForest<T> {
    fn free_ptrs_are_all_uniquie(&self) -> bool {
        let mut free_ptr_bucket = self.free_iter().collect::<Vec<_>>();
        free_ptr_bucket.sort();
        free_ptr_bucket
            .windows(2)
            .all(|window| window[1] != window[0])
    }
}

#[test]
fn can_alloc() {
    let mut tree = FlatForest::<i32>::new();
    let root = tree.allocate_node(10);
    println!("root data = {:?}", tree.get_data(root));
    assert_eq!(tree.get_data(root), &10);
    tree.free(root);
}

#[test]
fn can_free() {
    let mut tree = FlatForest::<_>::new();

    let nodes = (1..=50)
        .map(|val| tree.allocate_node((val + 9).to_string()))
        .collect::<Vec<_>>();

    nodes.into_iter().for_each(|node| tree.free(node));

    let freed_nodes = tree.free_iter().collect::<Vec<_>>();

    assert_eq!(freed_nodes.len(), 50);

    std::mem::drop(tree);
}

#[test]
fn can_drop() {
    let mut tree = FlatForest::<_>::new();

    let nodes = (1..=50)
        .map(|val| tree.allocate_node((val + 9).to_string()))
        .collect::<Vec<_>>();

    let mut dropped_nodes = vec![];
    tree.drain(|dropped_ptr| {
        dropped_nodes.push(dropped_ptr);
    });

    assert_eq!(dropped_nodes.len(), nodes.len());
    assert_eq!(nodes, dropped_nodes);
    assert_eq!(
        tree.needs_drop.populated_blocks().count(),
        0,
        "All bits should be set to zero after the drain"
    );
    assert!(
        tree.needs_drop
            .blocks_iter()
            .copied()
            .all(|block| block.bits() == 0),
        "needs_drop needs to be zero everywhere"
    );
    // IDEMPOTENCY CHECK ON `drain(..)`
    for _ in 0..50 {
        dropped_nodes.clear();
        tree.drain(|ptr| dropped_nodes.push(ptr));
        assert!(
            tree.is_freed.blocks_iter().all(|block| block.inner == !0),
            "All items should be flagged to be recycled"
        );
        assert_eq!(
            tree.freed_cache.len(),
            0,
            "cache should be flushed after a drain"
        );
        assert_eq!(dropped_nodes.len(), 0, "Drain should be idempotent");
    }

    std::mem::drop(tree);
}

#[test]
fn can_cache() {
    let mut tree = FlatForest::<i32>::new();
    let vals = [10, 15, 20, 30, 40, 50];
    let inserted_vals = vals
        .iter()
        .copied()
        .map(|e| tree.allocate_node(e))
        .collect::<Vec<_>>();

    tree.free(inserted_vals[0]);
    tree.free(inserted_vals.last().copied().unwrap());

    tree.update_freed_cache();
    let mut cached_values = vec![];
    while let Some(ptr) = tree.poll_cache() {
        cached_values.push(ptr);
    }
    assert_eq!(
        cached_values,
        vec![inserted_vals[0], inserted_vals.last().copied().unwrap()]
    );

    cached_values.clear();
    while let Some(ptr) = tree.poll_cache() {
        cached_values.push(ptr);
    }

    assert_eq!(cached_values, vec![], "Cache should be empty after polling");
    assert_eq!(
        tree.freed_cache.len(),
        0,
        "The cache's ring buffer should be empty"
    );
}

#[test]
fn forest_fuzz_simple() {
    use rand::Rng;
    const MAX_NODES: usize = 1024;
    const MIN_NODES: usize = 128;
    const MAX_ITERATIONS: usize = 32;
    let mut rand_ctx =
        <rand::rngs::SmallRng as rand::SeedableRng>::seed_from_u64(0xff0022aabbccddee);
    println!();
    for itcount in 0..MAX_ITERATIONS {
        let mut tree = FlatForest::new();
        let number_of_nodes =
            (rand_ctx.random::<u64>() as usize % (MAX_NODES - MIN_NODES)) + MIN_NODES;

        let max_frees = rand_ctx.random::<u64>() as usize % number_of_nodes;
        let mut free_count = 0;
        let mut nodes = (0..number_of_nodes)
            .map(|a| a.to_string())
            .map(|node_val| tree.allocate_node(node_val))
            .collect::<Vec<_>>();

        while !nodes.is_empty() && free_count < max_frees {
            let rand_idx = rand_ctx.random::<u64>() as usize % nodes.len();
            let node = nodes.remove(rand_idx);
            tree.free(node);
            free_count += 1;
        }

        assert_eq!(
            tree.data.len(),
            number_of_nodes,
            "The array should always match total number of initial frees "
        );

        assert_eq!(
            tree.free_iter().count(),
            free_count,
            "Free_count should match counting individual frees in the bitset",
        );

        assert!(
            tree.free_ptrs_are_all_uniquie(),
            "All freed pointers should be unique"
        );

        // NOTE: ALLOCATIONS PAST THIS POINT ARE EXPECTED TO BE RECYCLED
        // NO NEW NODE SHOULD BE ALLOCATED. WE ONLY ALLOCATE LESS THAN WE FREED TO
        // GUARANTEE FRESH NODES WONT BE CREATED.
        let num_node_recycles = (rand_ctx.random::<u64>() as usize) % free_count;
        for _ in 0..num_node_recycles {
            tree.allocate_node(rand_ctx.random::<u64>().to_string());
        }

        assert!(
            tree.free_ptrs_are_all_uniquie(),
            "All freed pointers should always be unique"
        );

        assert_eq!(
            tree.free_iter().count(),
            free_count - num_node_recycles,
            "The number of freed nodes should go down by num_nodes_recycled"
        );

        assert!(
            [tree.data.len(), tree.parent.len(), tree.children.len()]
                .iter()
                .copied()
                .all(|field_length| field_length == number_of_nodes),
            "if node_cache actually works properly, no new nodes expected"
        );

        println!("iter [{itcount}] completed <> nodes_alloc'd [{number_of_nodes}] <> nodes_recycled [{num_node_recycles}]");
    }
    //tree.free(nodes[2]);
}
