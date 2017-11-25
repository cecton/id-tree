use std::cmp::Ordering;
use std::marker::PhantomData;
use ::*;

use super::core::CoreTree;

///
/// A tree structure consisting of `OptNode`s.
///
/// # Panics
/// While it is highly unlikely, any function that takes a `NodeId` _can_ `panic`.  This, however,
/// should only happen due to improper `NodeId` management within `id_tree` and should have nothing
/// to do with the library user's code.
///
/// **If this ever happens please report the issue.** `Panic`s are not expected behavior for this
/// library, but they can happen due to bugs.
///
pub struct OptTree<'a, T: 'a> {
    pub(crate) core_tree: CoreTree<OptNode<T>, T>,
    pub(crate) phantom: PhantomData<&'a T>,
}

impl<'a, T> Tree<'a, T> for OptTree<'a, T> {
    type NodeType = OptNode<T>;
    type AncestorsIter = Ancestors<'a, OptTree<'a, T>, T>;
    type AncestorIdsIter = AncestorIds<'a, OptTree<'a, T>, T>;
    type ChildrenIter = OptChildren<'a, T>;
    type ChildrenIdsIter = OptChildrenIds<'a, T>;

    // todo: make real iterators for these.
    type PreOrderIter = Ancestors<'a, OptTree<'a, T>, T>;
    type PostOrderIter = Ancestors<'a, OptTree<'a, T>, T>;
    type LevelOrderIter = Ancestors<'a, OptTree<'a, T>, T>;

    fn new() -> Self {
        OptTreeBuilder::new().build()
    }

    fn height(&self) -> usize {
        unimplemented!()
    }

    fn insert(
        &mut self,
        node: OptNode<T>,
        behavior: InsertBehavior,
    ) -> Result<NodeId, NodeIdError> {
        match behavior {
            InsertBehavior::UnderNode(parent_id) => {
                self.core_tree.validate_node_id(parent_id)?;
                self.insert_under_node(node, parent_id)
            }
            InsertBehavior::AsRoot => Ok(self.set_root(node)),
        }
    }

    fn get(&self, node_id: &NodeId) -> Result<&OptNode<T>, NodeIdError> {
        self.core_tree.get(node_id)
    }

    fn get_mut(&mut self, node_id: &NodeId) -> Result<&mut OptNode<T>, NodeIdError> {
        self.core_tree.get_mut(node_id)
    }

    unsafe fn get_unchecked(&self, node_id: &NodeId) -> &OptNode<T> {
        self.core_tree
            .nodes
            .get_unchecked(node_id.index)
            .as_ref()
            .expect("Called VecTree.get_unchecked() with an invalid NodeId.")
    }

    unsafe fn get_unchecked_mut(&mut self, node_id: &NodeId) -> &mut OptNode<T> {
        self.core_tree
            .nodes
            .get_unchecked_mut(node_id.index)
            .as_mut()
            .expect("Called VecTree.get_unchecked_mut() with an invalid NodeId.")
    }

    fn remove(
        &mut self,
        node_id: NodeId,
        behavior: RemoveBehavior,
    ) -> Result<OptNode<T>, NodeIdError> {
        unimplemented!()
    }

    fn move_node(&mut self, node_id: &NodeId, behavior: MoveBehavior) -> Result<(), NodeIdError> {
        unimplemented!()
    }

    fn sort_children_by<F>(&mut self, node_id: &NodeId, compare: F) -> Result<(), NodeIdError>
    where
        F: FnMut(&OptNode<T>, &OptNode<T>) -> Ordering,
    {
        unimplemented!()
    }

    fn sort_children_by_data(&mut self, node_id: &NodeId) -> Result<(), NodeIdError>
    where
        T: Ord,
    {
        unimplemented!()
    }

    fn sort_children_by_key<K, F>(&mut self, node_id: &NodeId, f: F) -> Result<(), NodeIdError>
    where
        K: Ord,
        F: FnMut(&OptNode<T>) -> K,
    {
        unimplemented!()
    }

    fn swap_nodes(
        &mut self,
        first_id: &NodeId,
        second_id: &NodeId,
        behavior: SwapBehavior,
    ) -> Result<(), NodeIdError> {
        unimplemented!()
    }

    //todo: test
    fn root_node_id(&self) -> Option<&NodeId> {
        self.core_tree.root()
    }

    fn ancestors(&'a self, node_id: &NodeId) -> Result<Self::AncestorsIter, NodeIdError> {
        self.core_tree.validate_node_id(node_id)?;
        Ok(Ancestors::new(self, node_id.clone()))
    }

    fn ancestor_ids(&'a self, node_id: &NodeId) -> Result<Self::AncestorIdsIter, NodeIdError> {
        unimplemented!()
    }

    fn children(&'a self, node_id: &NodeId) -> Result<Self::ChildrenIter, NodeIdError> {
        unimplemented!()
    }

    fn children_ids(&'a self, node_id: &NodeId) -> Result<Self::ChildrenIdsIter, NodeIdError> {
        unimplemented!()
    }

    fn traverse_pre_order(&'a self, node_id: &NodeId) -> Result<Self::PreOrderIter, NodeIdError> {
        unimplemented!()
    }

    fn traverse_post_order(&'a self, node_id: &NodeId) -> Result<Self::PostOrderIter, NodeIdError> {
        unimplemented!()
    }

    fn traverse_level_order(
        &'a self,
        node_id: &NodeId,
    ) -> Result<Self::LevelOrderIter, NodeIdError> {
        unimplemented!()
    }
}

impl<'a, T> OptTree<'a, T> {
    ///
    ///
    ///
    fn insert_under_node(
        &mut self,
        mut node: OptNode<T>,
        parent_id: &NodeId,
    ) -> Result<NodeId, NodeIdError> {

        node.set_parent(Some(parent_id.clone()));

        let new_id = self.core_tree.insert(node);

        let children = {
            let parent = unsafe { self.get_unchecked(parent_id) };
            (parent.first_child().cloned(), parent.last_child().cloned())
        };

        match children {
            (Some(_), Some(last_id)) => {
                {
                    let parent = unsafe { self.get_unchecked_mut(parent_id) };
                    parent.set_last_child(Some(new_id.clone()));
                }

                {
                    let new_node = unsafe { self.get_unchecked_mut(&new_id) };
                    new_node.set_prev_sibling(Some(last_id.clone()));
                }

                let last_child = unsafe { self.get_unchecked_mut(&last_id) };
                last_child.set_next_sibling(Some(new_id.clone()));
            }
            //todo: find a better error message for these.
            (Some(_), None) => panic!("Found an OptNode in an invalid state."),
            (None, Some(_)) => panic!("Found an OptNode in an invalid state."),
            (None, None) => {
                let parent = unsafe { self.get_unchecked_mut(parent_id) };
                parent.set_first_child(Some(new_id.clone()));
                parent.set_last_child(Some(new_id.clone()));
            }
        }

        Ok(new_id)
    }

    ///
    /// Sets the root of the `Tree`.
    ///
    fn set_root(&mut self, new_root: OptNode<T>) -> NodeId {

        let current_root = self.core_tree.root.clone();
        let new_root_id = self.core_tree.set_root(new_root);

        if let Some(current_root_id) = current_root {
            {
                let current_root = unsafe { self.get_unchecked_mut(&current_root_id) };
                current_root.set_parent(Some(new_root_id.clone()));
            }

            let root = unsafe { self.get_unchecked_mut(&new_root_id) };
            root.set_first_child(Some(current_root_id.clone()));
            root.set_last_child(Some(current_root_id.clone()));
        }

        new_root_id
    }
}

#[cfg(test)]
mod opt_tree_tests {
    use ::*;
    use ::behaviors::InsertBehavior::*;

    fn new_tree<'a>() -> (NodeId, OptTree<'a, i32>) {
        let tree = OptTreeBuilder::new()
            .with_root(Node::new(1))
            .with_node_capacity(2usize)
            .with_swap_capacity(3usize)
            .build();

        (tree.core_tree.root.clone().unwrap(), tree)
    }

    #[test]
    fn new() {
        let tree: OptTree<i32> = OptTree::new();

        assert_eq!(tree.core_tree.root, None);
        assert_eq!(tree.core_tree.nodes.len(), 0);
        assert_eq!(tree.core_tree.free_ids.len(), 0);
    }

    #[test]
    fn get() {
        let (root_id, tree) = new_tree();

        let root = tree.get(&root_id).unwrap();

        assert_eq!(root.data(), &1);
    }

    #[test]
    fn get_mut() {
        let (root_id, mut tree) = new_tree();

        {
            let root = tree.get(&root_id).unwrap();
            assert_eq!(root.data(), &1);
        }

        {
            let root = tree.get_mut(&root_id).unwrap();
            assert_eq!(root.data(), &1);
            *root.data_mut() = 6;
            assert_eq!(root.data(), &6);
        }

        let root = tree.get(&root_id).unwrap();
        assert_eq!(root.data(), &6);
    }

    #[test]
    fn get_unchecked() {
        let (root_id, tree) = new_tree();

        let root = unsafe { tree.get_unchecked(&root_id) };

        assert_eq!(root.data(), &1);
    }

    #[test]
    fn get_unchecked_mut() {
        let (root_id, mut tree) = new_tree();

        {
            let root = unsafe { tree.get_unchecked(&root_id) };
            assert_eq!(root.data(), &1);
        }

        {
            let root = unsafe { tree.get_unchecked_mut(&root_id) };
            assert_eq!(root.data(), &1);
            *root.data_mut() = 6;
            assert_eq!(root.data(), &6);
        }

        let root = unsafe { tree.get_unchecked(&root_id) };
        assert_eq!(root.data(), &6);
    }

    #[test]
    fn insert() {
        let (root_id, mut tree) = new_tree();

        assert_eq!(tree.get(&root_id).unwrap().parent(), None);
        assert_eq!(tree.get(&root_id).unwrap().first_child(), None);
        assert_eq!(tree.get(&root_id).unwrap().last_child(), None);
        assert_eq!(tree.get(&root_id).unwrap().prev_sibling(), None);
        assert_eq!(tree.get(&root_id).unwrap().next_sibling(), None);

        let child_1 = tree.insert(Node::new(2), UnderNode(&root_id)).unwrap();

        assert_eq!(tree.get(&root_id).unwrap().parent(), None);
        assert_eq!(tree.get(&root_id).unwrap().first_child(), Some(&child_1));
        assert_eq!(tree.get(&root_id).unwrap().last_child(), Some(&child_1));
        assert_eq!(tree.get(&root_id).unwrap().prev_sibling(), None);
        assert_eq!(tree.get(&root_id).unwrap().next_sibling(), None);

        assert_eq!(tree.get(&child_1).unwrap().parent(), Some(&root_id));
        assert_eq!(tree.get(&child_1).unwrap().first_child(), None);
        assert_eq!(tree.get(&child_1).unwrap().last_child(), None);
        assert_eq!(tree.get(&child_1).unwrap().prev_sibling(), None);
        assert_eq!(tree.get(&child_1).unwrap().next_sibling(), None);

        let child_2 = tree.insert(Node::new(3), UnderNode(&root_id)).unwrap();

        assert_eq!(tree.get(&root_id).unwrap().parent(), None);
        assert_eq!(tree.get(&root_id).unwrap().first_child(), Some(&child_1));
        assert_eq!(tree.get(&root_id).unwrap().last_child(), Some(&child_2));
        assert_eq!(tree.get(&root_id).unwrap().prev_sibling(), None);
        assert_eq!(tree.get(&root_id).unwrap().next_sibling(), None);

        assert_eq!(tree.get(&child_1).unwrap().parent(), Some(&root_id));
        assert_eq!(tree.get(&child_1).unwrap().first_child(), None);
        assert_eq!(tree.get(&child_1).unwrap().last_child(), None);
        assert_eq!(tree.get(&child_1).unwrap().prev_sibling(), None);
        assert_eq!(tree.get(&child_1).unwrap().next_sibling(), Some(&child_2));

        assert_eq!(tree.get(&child_2).unwrap().parent(), Some(&root_id));
        assert_eq!(tree.get(&child_2).unwrap().first_child(), None);
        assert_eq!(tree.get(&child_2).unwrap().last_child(), None);
        assert_eq!(tree.get(&child_2).unwrap().prev_sibling(), Some(&child_1));
        assert_eq!(tree.get(&child_2).unwrap().next_sibling(), None);

        let new_root = tree.insert(Node::new(0), AsRoot).unwrap();
        let old_root = root_id;

        assert_eq!(tree.get(&old_root).unwrap().parent(), Some(&new_root));
        assert_eq!(tree.get(&old_root).unwrap().first_child(), Some(&child_1));
        assert_eq!(tree.get(&old_root).unwrap().last_child(), Some(&child_2));
        assert_eq!(tree.get(&old_root).unwrap().prev_sibling(), None);
        assert_eq!(tree.get(&old_root).unwrap().next_sibling(), None);

        assert_eq!(tree.get(&new_root).unwrap().parent(), None);
        assert_eq!(tree.get(&new_root).unwrap().first_child(), Some(&old_root));
        assert_eq!(tree.get(&new_root).unwrap().last_child(), Some(&old_root));
        assert_eq!(tree.get(&new_root).unwrap().prev_sibling(), None);
        assert_eq!(tree.get(&new_root).unwrap().next_sibling(), None);

        assert_eq!(tree.get(&child_1).unwrap().parent(), Some(&old_root));
        assert_eq!(tree.get(&child_1).unwrap().first_child(), None);
        assert_eq!(tree.get(&child_1).unwrap().last_child(), None);
        assert_eq!(tree.get(&child_1).unwrap().prev_sibling(), None);
        assert_eq!(tree.get(&child_1).unwrap().next_sibling(), Some(&child_2));

        assert_eq!(tree.get(&child_2).unwrap().parent(), Some(&old_root));
        assert_eq!(tree.get(&child_2).unwrap().first_child(), None);
        assert_eq!(tree.get(&child_2).unwrap().last_child(), None);
        assert_eq!(tree.get(&child_2).unwrap().prev_sibling(), Some(&child_1));
        assert_eq!(tree.get(&child_2).unwrap().next_sibling(), None);
    }
}