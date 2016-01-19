use std::collections::vec_deque::Drain;
use std::collections::vec_deque::Iter;
use std::collections::vec_deque::IterMut;
use std::collections::VecDeque;

use std::fmt::Debug;

pub struct ScoredCircularBuffer<K,V> {
	size: usize,
	deque: VecDeque<ScoredItem<K,V>>,
//	rand_factor: Option<f64>,
	sort: Sort
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Sort {
	Ascending,
	Descending
}

pub trait Scored<K> {
	fn score(&self) -> K;
}

#[derive(Clone)]
pub struct ScoredItem<K, V> {
	pub score: K,
	pub value: V
}

#[allow(dead_code)]
impl<K: PartialOrd<K> + Debug, V:Scored<K>> ScoredCircularBuffer<K,V> {
	pub fn push_scored( &mut self, item: V ) {
		let score = item.score();
		self.push( item, score );
	}
}

impl<K: PartialOrd<K> + Debug, V> ScoredCircularBuffer<K,V> {	
	pub fn push_bucket<F, B:PartialEq<B>>( &mut self, value: V, score: K, bucket_fn: F ) 
		where F: Fn(&V) -> B {
		let len = self.deque.len();
		let new = ScoredItem { score: score, value: value };
		
		for _ in 0..len {
			let compare = self.deque.pop_back().unwrap();
			
			if bucket_fn( &compare.value ) == bucket_fn( &new.value ) {
				let keep = match self.sort {
					Sort::Descending => new.score < compare.score,
					Sort::Ascending => new.score > compare.score
				};
				
				if keep {
//					println!("Compared {:?} to {:?}, keeping #1", compare.score, new.score );
					self.deque.push_front( compare );
				} else {
//					println!("Compared {:?} to {:?}, keeping #2", compare.score, new.score );
					self.deque.push_front( new );
				}
				
				return;
			} else {
				self.deque.push_front(compare);
			}
		}
		
		self.push_scored_item(new);
	}
}

#[allow(dead_code)]
impl<K: PartialOrd<K> + Debug, V> ScoredCircularBuffer<K,V> {
	pub fn new(size: usize, sort: Sort ) -> ScoredCircularBuffer<K,V> {
		ScoredCircularBuffer { 
			size: size, 
			deque: VecDeque::with_capacity( size ), 
//			rand_factor: None,
			sort: sort 
		}
	}
	
	pub fn capacity( &self ) -> usize {
		self.size
	}
	
	pub fn len( &self ) -> usize {
		self.deque.len()
	}
	
	pub fn is_empty( &self ) -> bool {
		self.deque.is_empty()
	}
	
	pub fn drain( &mut self ) -> Drain<ScoredItem<K,V>> {
		self.deque.drain(..)
	}
	
	pub fn clear( &mut self ) {
		self.deque.clear();
	}
	
	pub fn push( &mut self, value: V, score: K ) {
		self.push_scored_item( ScoredItem { score: score, value: value } );
	}
	
	fn push_scored_item( &mut self, new: ScoredItem<K,V> ) {
		let mut remove = new;
		let len = self.deque.len();
		if len < self.size || len == 0 {
			self.deque.push_front( remove );
			return;
		}
		
		for _ in 0..len {
			let compare = self.deque.pop_back().unwrap();
			let keep = match self.sort {
				Sort::Descending => remove.score < compare.score,
				Sort::Ascending => remove.score > compare.score
			};
			
			if keep {
				self.deque.push_front( compare );
			} else {
				self.deque.push_front( remove );
				remove = compare;
			}
		}
	}
	
	pub fn push_opt( &mut self, value: V, score: Option<K> ) {
		match score {
			Some( k ) => self.push( value, k ),
			None => {}
		}
	}
	
	pub fn append( &mut self, other: &mut ScoredCircularBuffer<K,V> )  {
		for elem in other.drain() {
			self.push( elem.value, elem.score );
		}
	}
	
//	pub fn first(&self) -> Option<&V> {
//		let vals = self.sort();
//		vals.drain(..).next()
//	}
//	
//	pub fn first_mut(&mut self) -> Option<V> {
//		let vals = self.sort_mut();
//		vals.drain(..).next()
//	}
//		
//	pub fn last(&self) -> Option<&V> {
//		let vals = self.sort();
//		vals.drain(..).last()
//	}
//	
//	pub fn last_mut(&mut self) -> Option<V> {
//		let vals = self.sort_mut();
//		vals.drain(..).last()
//	}
	
	pub fn iter(&self) -> Iter<ScoredItem<K,V>> {
		self.deque.iter()
	}
	
	pub fn iter_mut(&mut self) -> IterMut<ScoredItem<K,V>> {
		self.deque.iter_mut()
	}
	
	pub fn sort( &self ) -> Vec<&V> {
		let mut sorted: Vec<&ScoredItem<K,V>> = self.iter().collect();
		
		match self.sort {
			Sort::Ascending => sorted.sort_by(|ref a, ref b| a.score.partial_cmp( &b.score ).expect("Failed to compare ScoredCircularBuffer score") ),
			Sort::Descending => sorted.sort_by(|ref a, ref b| b.score.partial_cmp( &a.score ).expect("Failed to compare ScoredCircularBuffer score") )
		}
		
		let result : Vec<&V> = sorted.drain(..).map(|e| &e.value ).collect();
		result
	}

	pub fn sort_mut( &mut self ) -> Vec<V> {
		let mut sorted: Vec<ScoredItem<K,V>> = self.drain().collect();
		
		match self.sort {
			Sort::Ascending => sorted.sort_by(|ref a, ref b| a.score.partial_cmp( &b.score ).expect("Failed to compare ScoredCircularBuffer score") ),
			Sort::Descending => sorted.sort_by(|ref a, ref b| b.score.partial_cmp( &a.score ).expect("Failed to compare ScoredCircularBuffer score") )
		}
		
		let result : Vec<V> = sorted.drain(..).map(|e| e.value ).collect();
		result
	}
}

mod tests {
	use super::*;
	
	fn new_buffer_desc() -> ScoredCircularBuffer<usize, usize> {
		let mut buffer = ScoredCircularBuffer::new(2, Sort::Descending);
		buffer.push( 1, 1 );
		buffer.push( 3, 3 );
		buffer.push( 5, 5 );
		
		buffer.push( 2, 2 );
		buffer.push( 4, 4 );
		buffer.push( 6, 6 );
		
		buffer
	}
	
	fn new_buffer_asc() -> ScoredCircularBuffer<usize, usize> {
		let mut buffer = ScoredCircularBuffer::new(2, Sort::Ascending);
		buffer.push( 6, 6 );
		buffer.push( 4, 4 );
		buffer.push( 2, 2 );
		
		buffer.push( 5, 5 );
		buffer.push( 3, 3 );
		buffer.push( 1, 1 );
		
		buffer
	}
	
	#[derive(PartialEq, Debug, Clone)]
	struct BucketTest {
		val: usize,
		bkt: usize
	}
	
	fn new_buffer_bucketed( sort: Sort ) -> ScoredCircularBuffer<usize, BucketTest> {
		let mut buffer = ScoredCircularBuffer::new(2, sort);
		
		buffer.push_bucket( BucketTest{val: 4, bkt: 1}, 4, |e| e.bkt );
		buffer.push_bucket( BucketTest{val: 2, bkt: 1}, 2, |e| e.bkt );
		buffer.push_bucket( BucketTest{val: 1, bkt: 1}, 1, |e| e.bkt );
		
		buffer.push_bucket( BucketTest{val: 5, bkt: 2}, 5, |e| e.bkt );
		buffer.push_bucket( BucketTest{val: 3, bkt: 2}, 3, |e| e.bkt );
		
		buffer
	}
	
	#[test]
	fn test_sort_desc() {
		let buffer_desc = new_buffer_desc();
		let mut desc_sort = buffer_desc.sort();
		let mut desc_sort_iter = desc_sort.drain(..);
		assert_eq!( desc_sort_iter.next(), Some(&6) );
		assert_eq!( desc_sort_iter.next(), Some(&5) );
		assert_eq!( desc_sort_iter.next(), None );
    }
	
	#[test]
	fn test_sort_mut_desc() {
		let mut buffer_desc = new_buffer_desc();
		let mut desc_sort = buffer_desc.sort_mut();
		let mut desc_sort_iter = desc_sort.drain(..);
		assert_eq!( desc_sort_iter.next(), Some(6) );
		assert_eq!( desc_sort_iter.next(), Some(5) );
		assert_eq!( desc_sort_iter.next(), None );
    }
	
	#[test]
	fn test_sort_asc() {
		let buffer_desc = new_buffer_asc();
		let mut desc_sort = buffer_desc.sort();
		let mut desc_sort_iter = desc_sort.drain(..);
		assert_eq!( desc_sort_iter.next(), Some(&1) );
		assert_eq!( desc_sort_iter.next(), Some(&2) );
		assert_eq!( desc_sort_iter.next(), None );
    }
	
	#[test]
	fn test_sort_mut_asc() {
		let mut buffer_desc = new_buffer_asc();
		let mut desc_sort = buffer_desc.sort_mut();
		let mut desc_sort_iter = desc_sort.drain(..);
		assert_eq!( desc_sort_iter.next(), Some(1) );
		assert_eq!( desc_sort_iter.next(), Some(2) );
		assert_eq!( desc_sort_iter.next(), None );
    }
	
	#[test]
	fn test_sort_bucket_asc() {
		let buffer_desc = new_buffer_bucketed( Sort::Ascending );
		let mut desc_sort = buffer_desc.sort();
		let mut desc_sort_iter = desc_sort.drain(..);
		assert_eq!( desc_sort_iter.next(), Some(&BucketTest{ val: 1, bkt: 1}) );
		assert_eq!( desc_sort_iter.next(), Some(&BucketTest{ val: 3, bkt: 2}) );
		assert_eq!( desc_sort_iter.next(), None );
    }
	
	#[test]
	fn test_sort_bucket_desc() {
		let buffer_desc = new_buffer_bucketed( Sort::Descending );
		let mut desc_sort = buffer_desc.sort();
		let mut desc_sort_iter = desc_sort.drain(..);
		assert_eq!( desc_sort_iter.next(), Some(&BucketTest{ val: 5, bkt: 2}) );
		assert_eq!( desc_sort_iter.next(), Some(&BucketTest{ val: 4, bkt: 1}) );
		assert_eq!( desc_sort_iter.next(), None );
    }
}