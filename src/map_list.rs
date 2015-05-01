use core::hash::Hash;

use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut, Drain, Entry, Keys, Values};

type ValueList<E> = Vec<E>;

pub struct MapList<K,E> {
	map: HashMap<K,ValueList<E>>
}

#[allow(dead_code)]
impl<K: Eq+Hash, E> MapList<K,E> {
	pub fn new() -> MapList<K,E> {
		MapList { 
			map: HashMap::new()
		}
	}
	
	pub fn with_capacity( capacity: usize ) -> MapList<K,E> {
		MapList { 
			map: HashMap::with_capacity( capacity )
		}
	}
	
	pub fn with_hashmap( provided_map: HashMap<K,Vec<E>> ) -> MapList<K,E> {
		MapList { 
			map: provided_map
		}
	}
	
	pub fn capacity( &self ) -> usize {
		self.map.capacity()
	}
	
	pub fn keys<'a>( &'a self ) -> Keys<'a, K, ValueList<E>> {
		self.map.keys()
	}
	
	pub fn values<'a>( &'a self ) -> Values<'a, K, ValueList<E>> {
		self.map.values()
	}
	
	pub fn iter( &self ) -> Iter<K,ValueList<E>> {
		self.map.iter()
	}
	
	pub fn iter_mut( &mut self ) -> IterMut<K,ValueList<E>> {
		self.map.iter_mut()
	}
	
	pub fn entry( &mut self, key: K ) -> Entry<K,ValueList<E>> {
		self.map.entry( key )
	}
	
	pub fn len( &self ) -> usize {
		self.map.len()
	}
	
	pub fn is_empty( &self ) -> bool {
		self.map.is_empty()
	}
	
	pub fn drain( &mut self ) -> Drain<K, ValueList<E>> {
		self.map.drain()
	}
	
	pub fn clear( &mut self ) {
		self.map.clear()
	}
	
	pub fn get( &self, key: &K ) -> Option<&ValueList<E>> {
		self.map.get( key )
	}
	
	pub fn get_mut( &mut self, key: &K ) -> Option<&mut ValueList<E>> {
		self.map.get_mut( key )
	}
	
	pub fn insert( &mut self, key: K, elem: E ) {
		match self.get_mut( &key ) {
			Some(mut val) => {
				val.push( elem );
				return;
			},
			None => {}
		}
		
		let mut list = ValueList::new();
		list.push( elem );
		self.insert_list( key, list );
	}
	
	pub fn insert_list( &mut self, key: K, list: ValueList<E> ) -> Option<ValueList<E>> {
		self.map.insert( key, list )
	}
	
	pub fn remove( &mut self, key: &K ) -> Option<ValueList<E>> {
		self.map.remove( key )
	}
}