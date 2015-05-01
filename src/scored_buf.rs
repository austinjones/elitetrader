use std::collections::vec_deque::Drain;
use std::collections::vec_deque::Iter;
use std::collections::vec_deque::IterMut;
use std::collections::VecDeque;

use std::fmt::Debug;

pub trait Scored<K> {
	fn get_score(&self) -> K;
}

struct ScoredItem<K, V> {
	score: K,
	value: V
}

pub struct ScoredCircularBuffer<K,V> {
	size: usize,
	deque: VecDeque<ScoredItem<K,V>>,
	sort: Sort
}

#[allow(dead_code)]
pub enum Sort {
	Ascending,
	Descending
}

#[allow(dead_code)]
impl<K: PartialOrd<K> + Debug, V:Scored<K>> ScoredCircularBuffer<K,V> {
	pub fn push_scored( &mut self, item: V ) {
		let score = item.get_score();
		self.push( item, score );
	}
}

#[allow(dead_code)]
impl<K: PartialOrd<K> + Debug, V> ScoredCircularBuffer<K,V> {
	pub fn new(size: usize, sort: Sort ) -> ScoredCircularBuffer<K,V> {
		ScoredCircularBuffer { size: size, deque: VecDeque::with_capacity( size ), sort: sort }
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
	
	fn drain( &mut self ) -> Drain<ScoredItem<K,V>> {
		self.deque.drain()
	}
	
	pub fn clear( &mut self ) {
		self.deque.clear();
	}
	
	pub fn push( &mut self, value: V, score: K ) -> bool {
		let len = self.deque.len();
		if len < self.size || len == 0 {
			self.deque.push_front( ScoredItem{ score: score, value: value }  );
			return true;
		}
		
		let compare = self.deque.pop_back().unwrap();
		let outlier = match self.sort {
			Sort::Descending => score > compare.score,
			Sort::Ascending => score < compare.score
		};
		
		let add = match outlier {
			true => ScoredItem { score: score, value: value },
			false => compare
		};
		
		self.deque.push_front( add );
		
		outlier
	}
	
	pub fn push_opt( &mut self, value: V, score: Option<K> ) -> bool {
		match score {
			Some( k ) => self.push( value, k ),
			None => false
		}
	}
	
	pub fn append( &mut self, other: &mut ScoredCircularBuffer<K,V> )  {
		for elem in other.drain() {
			self.push( elem.value, elem.score );
		}
	}
	
	
	fn iter(&self) -> Iter<ScoredItem<K,V>> {
		self.deque.iter()
	}
	
	fn iter_mut(&mut self) -> IterMut<ScoredItem<K,V>> {
		self.deque.iter_mut()
	}
	
	pub fn sort( &self ) -> Vec<&V> {
		let mut sorted: Vec<&ScoredItem<K,V>> = self.iter().collect();
		// sort descending
		match self.sort {
			Sort::Ascending => sorted.sort_by(|a,b| a.score.partial_cmp( &b.score ).unwrap() ),
			Sort::Descending => sorted.sort_by(|a,b| b.score.partial_cmp( &a.score ).unwrap() )
		}
		
		let result : Vec<&V> = sorted.drain().map(|e| &e.value ).collect();
		result
	}

	pub fn sort_mut( &mut self ) -> Vec<V> {
		let mut sorted: Vec<ScoredItem<K,V>> = self.drain().collect();
		// sort descending
		
		match self.sort {
			Sort::Ascending => sorted.sort_by(|a,b| a.score.partial_cmp( &b.score ).unwrap() ),
			Sort::Descending => sorted.sort_by(|a,b| b.score.partial_cmp( &a.score ).unwrap() )
		}
		
		let result : Vec<V> = sorted.drain().map(|e| e.value ).collect();
		result
	}
}