use super::*;

#[derive(Copy, Clone, Debug)]
struct UnitTestTag;

#[test]
fn can_alloc() {
    let mut tree = FlatForest::<UnitTestTag, i32>::new();
    let root = tree.allocate_node(10);
    println!("root data = {:?}", tree.get_data(root));
    assert_eq!(tree.get_data(root), &10);

    tree.free(root);

    assert_eq!(tree.free_pool_header, root);
}

#[test]
fn can_free() {
    let mut tree = FlatForest::<UnitTestTag, _>::new();

    let nodes = (1..=50)
        .map(|val| tree.allocate_node((val + 9).to_string()))
        .collect::<Vec<_>>();

    nodes.into_iter().for_each(|node| tree.free(node));

    let freed_nodes = tree.free_iter().collect::<Vec<_>>();
    // println!("freed_nodes = [{:?}]", freed_nodes);

    assert_eq!(freed_nodes.len(), 50);

    std::mem::drop(tree);
}

#[test]
fn can_drop() {
    let mut tree = FlatForest::<UnitTestTag, _>::new();

    let nodes = (1..=50)
        .map(|val| tree.allocate_node((val + 9).to_string()))
        .collect::<Vec<_>>();

    std::mem::drop(tree);
}
#[test]
fn can_drop_extended() {}
