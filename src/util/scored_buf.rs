use std::collections::vec_deque::Drain;
use std::collections::vec_deque::Iter;
use std::collections::vec_deque::IterMut;
use std::collections::VecDeque;

use std::fmt::Debug;

pub struct ScoredCircularBuffer<K, V> {
    size: usize,
    deque: VecDeque<ScoredItem<K, V>>,
    sort: Sort,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Sort {
    Ascending,
    Descending,
}

impl Sort {
    pub fn gt<K: PartialOrd<K>, V>(&self, v1: &ScoredItem<K, V>, v2: &ScoredItem<K, V>) -> bool {
        match *self {
            Sort::Descending => v1.score > v2.score,
            Sort::Ascending => v1.score < v2.score,
        }
    }
}

pub trait Scored<K> {
    fn score(&self) -> K;
}

#[derive(Clone)]
pub struct ScoredItem<K, V> {
    pub score: K,
    pub value: V,
}

#[allow(dead_code)]
impl<K: PartialOrd<K> + Debug, V: Scored<K>> ScoredCircularBuffer<K, V> {
    pub fn push_scored(&mut self, item: V) {
        let score = item.score();
        self.push(item, score);
    }
}

impl<K: PartialOrd<K> + Debug, V> ScoredCircularBuffer<K, V> {
    pub fn push_bucket<F, B: PartialEq<B>>(&mut self, value: V, score: K, bucket_fn: F)
    where
        F: Fn(&V) -> B,
    {
        let scored_item = ScoredItem {
            score: score,
            value: value,
        };

        enum PushBucketAction {
            Accept,
            Replace(usize),
            Discard,
        }

        let action = match self
            .deque
            .iter()
            .enumerate()
            .find(|&(_, compare)| bucket_fn(&compare.value) == bucket_fn(&scored_item.value))
        {
            Some((index, compare)) => {
                if self.sort.gt(&scored_item, compare) {
                    //					println!("Replacing {:?} with {:?}", compare.score, scored_item.score);
                    PushBucketAction::Replace(index)
                } else {
                    //					println!("Discarding - keep: {:?} and discarding: {:?}", compare.score, scored_item.score);
                    PushBucketAction::Discard
                }
            }
            None => {
                //				println!("Accepting: {:?}", scored_item.score);
                PushBucketAction::Accept
            }
        };

        match action {
            PushBucketAction::Accept => self.push_scored_item(scored_item),
            PushBucketAction::Replace(removal_index) => {
                self.deque.remove(removal_index);
                self.push_scored_item(scored_item);
            }
            PushBucketAction::Discard => {}
        }
    }
}

#[allow(dead_code)]
impl<K: PartialOrd<K> + Debug, V> ScoredCircularBuffer<K, V> {
    pub fn new(size: usize, sort: Sort) -> ScoredCircularBuffer<K, V> {
        ScoredCircularBuffer {
            size: size,
            deque: VecDeque::with_capacity(size + 1),
            //			rand_factor: None,
            sort: sort,
        }
    }

    pub fn capacity(&self) -> usize {
        self.size
    }

    pub fn len(&self) -> usize {
        self.deque.len()
    }

    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }

    pub fn drain(&mut self) -> Drain<ScoredItem<K, V>> {
        self.deque.drain(..)
    }

    pub fn clear(&mut self) {
        self.deque.clear();
    }

    pub fn push(&mut self, value: V, score: K) {
        self.push_scored_item(ScoredItem {
            score: score,
            value: value,
        });
    }

    fn insert_index(&self, new: &ScoredItem<K, V>) -> usize {
        for (i, compare) in self.deque.iter().enumerate() {
            if self.sort.gt(compare, &new) {
                return i;
            }
        }

        return self.deque.len();
    }

    fn push_scored_item(&mut self, new: ScoredItem<K, V>) {
        if self.deque.len() == 0 {
            self.deque.push_front(new);
            return;
        }

        let insert_index = self.insert_index(&new);
        self.deque.insert(insert_index, new);

        if self.deque.len() > self.size {
            let _popped = self.deque.pop_front().unwrap();
        }
    }

    pub fn push_opt(&mut self, value: V, score: Option<K>) {
        match score {
            Some(k) => self.push(value, k),
            None => {}
        }
    }

    pub fn append(&mut self, other: &mut ScoredCircularBuffer<K, V>) {
        for elem in other.drain() {
            self.push(elem.value, elem.score);
        }
    }

    pub fn iter(&self) -> Iter<ScoredItem<K, V>> {
        self.deque.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<ScoredItem<K, V>> {
        self.deque.iter_mut()
    }

    pub fn sort(&self) -> Vec<&V> {
        self.iter().map(|e| &e.value).rev().collect()
    }

    pub fn sort_mut(&mut self) -> Vec<V> {
        self.drain().map(|e| e.value).rev().collect()
    }
}

mod tests {
    use super::*;

    fn new_buffer_desc() -> ScoredCircularBuffer<usize, usize> {
        let mut buffer = ScoredCircularBuffer::new(2, Sort::Descending);
        buffer.push(1, 1);
        buffer.push(3, 3);
        buffer.push(5, 5);

        buffer.push(2, 2);
        buffer.push(4, 4);
        buffer.push(6, 6);

        buffer
    }

    fn new_buffer_asc() -> ScoredCircularBuffer<usize, usize> {
        let mut buffer = ScoredCircularBuffer::new(2, Sort::Ascending);
        buffer.push(6, 6);
        buffer.push(4, 4);
        buffer.push(2, 2);

        buffer.push(5, 5);
        buffer.push(3, 3);
        buffer.push(1, 1);

        buffer
    }

    #[derive(PartialEq, Debug, Clone)]
    struct BucketTest {
        val: usize,
        bkt: usize,
    }

    fn new_buffer_bucketed(sort: Sort) -> ScoredCircularBuffer<usize, BucketTest> {
        let mut buffer = ScoredCircularBuffer::new(2, sort);

        buffer.push_bucket(BucketTest { val: 4, bkt: 1 }, 4, |e| e.bkt);
        buffer.push_bucket(BucketTest { val: 2, bkt: 1 }, 2, |e| e.bkt);
        buffer.push_bucket(BucketTest { val: 1, bkt: 1 }, 1, |e| e.bkt);

        buffer.push_bucket(BucketTest { val: 5, bkt: 2 }, 5, |e| e.bkt);
        buffer.push_bucket(BucketTest { val: 3, bkt: 2 }, 3, |e| e.bkt);

        buffer
    }

    #[test]
    fn test_sort_desc() {
        let buffer_desc = new_buffer_desc();
        let mut desc_sort = buffer_desc.sort();
        println!("{:?}", desc_sort);
        let mut desc_sort_iter = desc_sort.drain(..);
        assert_eq!(desc_sort_iter.next(), Some(&6));
        assert_eq!(desc_sort_iter.next(), Some(&5));
        assert_eq!(desc_sort_iter.next(), None);
    }

    #[test]
    fn test_sort_mut_desc() {
        let mut buffer_desc = new_buffer_desc();
        let mut desc_sort = buffer_desc.sort_mut();
        println!("{:?}", desc_sort);
        let mut desc_sort_iter = desc_sort.drain(..);
        assert_eq!(desc_sort_iter.next(), Some(6));
        assert_eq!(desc_sort_iter.next(), Some(5));
        assert_eq!(desc_sort_iter.next(), None);
    }

    #[test]
    fn test_sort_asc() {
        let buffer_desc = new_buffer_asc();
        let mut desc_sort = buffer_desc.sort();
        let mut desc_sort_iter = desc_sort.drain(..);
        assert_eq!(desc_sort_iter.next(), Some(&1));
        assert_eq!(desc_sort_iter.next(), Some(&2));
        assert_eq!(desc_sort_iter.next(), None);
    }

    #[test]
    fn test_sort_mut_asc() {
        let mut buffer_desc = new_buffer_asc();
        let mut desc_sort = buffer_desc.sort_mut();
        let mut desc_sort_iter = desc_sort.drain(..);
        assert_eq!(desc_sort_iter.next(), Some(1));
        assert_eq!(desc_sort_iter.next(), Some(2));
        assert_eq!(desc_sort_iter.next(), None);
    }

    #[test]
    fn test_sort_bucket_asc() {
        let buffer_desc = new_buffer_bucketed(Sort::Ascending);
        let mut desc_sort = buffer_desc.sort();
        let mut desc_sort_iter = desc_sort.drain(..);
        assert_eq!(desc_sort_iter.next(), Some(&BucketTest { val: 1, bkt: 1 }));
        assert_eq!(desc_sort_iter.next(), Some(&BucketTest { val: 3, bkt: 2 }));
        assert_eq!(desc_sort_iter.next(), None);
    }

    #[test]
    fn test_sort_bucket_desc() {
        let buffer_desc = new_buffer_bucketed(Sort::Descending);
        let mut desc_sort = buffer_desc.sort();
        let mut desc_sort_iter = desc_sort.drain(..);
        assert_eq!(desc_sort_iter.next(), Some(&BucketTest { val: 5, bkt: 2 }));
        assert_eq!(desc_sort_iter.next(), Some(&BucketTest { val: 4, bkt: 1 }));
        assert_eq!(desc_sort_iter.next(), None);
    }
}
