//! Kernel tree vector implementation.
//! It is a tree-like structure that contains vectors of constant size as leaves.
//! This is useful to avoid reallocating memory when the vector's capacity is reached.
//!
//! TODO: this implementation is incomplete

use crate::{
    kcell::KCell,
    kmalloc::{Error, KMalloc},
    krc::KRefCell,
    kvec::{KVec, KVecError},
};

pub enum KTreeError {
    EmptyInternal,
    UnorderedKeys,
    KMalloc(Error),
    KVec(KVecError),
}

impl From<Error> for KTreeError {
    fn from(error: Error) -> Self {
        KTreeError::KMalloc(error)
    }
}

impl From<KVecError> for KTreeError {
    fn from(error: KVecError) -> Self {
        KTreeError::KVec(error)
    }
}

/// Kernel tree vector
/// It is a tree-like structure that contains vectors of constant size as leaves.
/// This is useful to avoid reallocating memory when the vector's capacity is reached.
///
/// This tree maintains a doubly linked list of leaves to allow for efficient linear traversal of the tree.
///
/// Finding an element in the tree is O(log (n / k)), where k is the number of elements in each leaf.
/// Inserting at a random position should be O(log (n / k)). At the end, or at the beginning, it should be O(1).
/// Deleting an element should be O(log (n / k)). Deleting an element in the middle of the tree amounts to changing it to None.
///
/// Once a full vector is emptied, the leaf should be removed from the tree and coalesced with the parent.
pub struct KTreeVec<T: 'static, const K: usize> {
    /// The root node of the B-Tree.
    root: KTreeVecNode<T, K>,
    /// The leftmost leaf of the B-Tree.
    leftmost_leaf: KRefCell<KTreeLeaf<T>>,
    /// The rightmost leaf of the B-Tree.
    rightmost_leaf: KRefCell<KTreeLeaf<T>>,
}

pub enum KTreeVecNode<T: 'static, const K: usize> {
    Internal(KRefCell<KTreeInternal<T, K>>),
    Leaf(KRefCell<KTreeLeaf<T>>),
}

impl<T: 'static + Ord, const K: usize> KTreeVecNode<T, K> {
    fn insert(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), KTreeError> {
        match self {
            KTreeVecNode::Internal(internal) => internal.get_mut().insert(kbox_maker, value),
            KTreeVecNode::Leaf(leaf) => leaf.get_mut().insert(kbox_maker, value),
        }
    }
}

pub struct KTreeInternalKV<T: 'static, const K: usize> {
    key: T,
    // All the keys in child are greater than or equal to the key.
    child: KTreeVecNode<T, K>,
}

pub struct KTreeInternal<T: 'static, const K: usize>(KVec<KTreeInternalKV<T, K>>);

impl<T: 'static + Ord, const K: usize> KTreeInternal<T, K> {
    fn insert(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), KTreeError> {
        // Find the index of the child to insert the value into.
        let values = &mut self.0;
        let max_key = values.last_mut().ok_or(KTreeError::EmptyInternal)?;

        if value >= max_key.key {
            max_key.child.insert(kbox_maker, value)?;
        } else {
            // Find the index of the child to insert the value into.
            let index = values
                .iter()
                .position(|k| k.key >= value)
                .ok_or(KTreeError::UnorderedKeys)?;

            values
                .get_mut(index.saturating_sub(1))
                .unwrap()
                .child
                .insert(kbox_maker, value)?;
        }

        // TODO: split the internal node if it is full, rebalance the tree if necessary
        todo!()
    }
}

pub struct KTreeLeaf<T: 'static> {
    values: KVec<T>,
    next: Option<KRefCell<KTreeLeaf<T>>>,
    prev: Option<KRefCell<KTreeLeaf<T>>>,
}

impl<T: 'static + Ord> KTreeLeaf<T> {
    fn insert(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), KTreeError> {
        let Some(last) = self.values.last_mut() else {
            self.values.push_no_resize(value)?;
            return Ok(());
        };

        if value >= *last {
            self.values.push_no_resize(value)?;
            return Ok(());
        }

        // Find the index of the value to insert into.
        let index = self
            .values
            .iter()
            .position(|v| v >= &value)
            .ok_or(KTreeError::UnorderedKeys)?;
        self.values.insert(index, value)?;

        Ok(())
    }
}

impl<T: 'static + Ord, const K: usize> KTreeVec<T, K> {
    pub fn new(kbox_maker: &mut KMalloc) -> Result<Self, Error> {
        let leaf = KVec::new(10, kbox_maker)?;
        let leaf_node = kbox_maker.new_rc(KCell::from(KTreeLeaf {
            values: leaf,
            next: None,
            prev: None,
        }))?;

        Ok(Self {
            root: KTreeVecNode::Leaf(leaf_node.clone()),
            leftmost_leaf: leaf_node.clone(),
            rightmost_leaf: leaf_node.clone(),
        })
    }

    pub fn insert(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), KTreeError> {
        // Recursively traverse the tree to find the correct leaf to insert the value into.
        match &mut self.root {
            KTreeVecNode::Internal(internal) => {
                internal.get_mut().insert(kbox_maker, value)?;
            }
            KTreeVecNode::Leaf(leaf) => {
                leaf.get_mut().insert(kbox_maker, value)?;
            }
        }

        Ok(())
    }
}
