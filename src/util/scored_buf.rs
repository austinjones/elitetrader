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
pub enum Sort {
	Ascending,
	Descending
}

pub trait Scored<K> {
	fn score(&self) -> K;
}

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
		self.deque.drain()
	}
	
	pub fn clear( &mut self ) {
		self.deque.clear();
	}
	
	pub fn push( &mut self, value: V, score: K ) {
		let len = self.deque.len();
		let mut remove = ScoredItem { score: score, value: value };
		
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
//		vals.drain().next()
//	}
//	
//	pub fn first_mut(&mut self) -> Option<V> {
//		let vals = self.sort_mut();
//		vals.drain().next()
//	}
//		
//	pub fn last(&self) -> Option<&V> {
//		let vals = self.sort();
//		vals.drain().last()
//	}
//	
//	pub fn last_mut(&mut self) -> Option<V> {
//		let vals = self.sort_mut();
//		vals.drain().last()
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
		
		let result : Vec<&V> = sorted.drain().map(|e| &e.value ).collect();
		result
	}

	pub fn sort_mut( &mut self ) -> Vec<V> {
		let mut sorted: Vec<ScoredItem<K,V>> = self.drain().collect();
		
		match self.sort {
			Sort::Ascending => sorted.sort_by(|ref a, ref b| a.score.partial_cmp( &b.score ).expect("Failed to compare ScoredCircularBuffer score") ),
			Sort::Descending => sorted.sort_by(|ref a, ref b| b.score.partial_cmp( &a.score ).expect("Failed to compare ScoredCircularBuffer score") )
		}
		
		let result : Vec<V> = sorted.drain().map(|e| e.value ).collect();
		result
	}
}
