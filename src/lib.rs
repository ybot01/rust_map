#![no_std]
extern crate alloc;

use alloc::boxed::Box;

#[derive(Debug, PartialEq, Eq)]
pub enum ByteArrayTreeMap<const N: usize, V>{
    Branch([Box<ByteArrayTreeMap<N, V>>; 2]),
    Leaf(Option<([u8;N], V)>)
}

impl<const N: usize, V> ByteArrayTreeMap<N, V>{

    const fn get_index(key: &[u8;N], depth: usize) -> usize{
        //((key[depth / 4] >> (6 - ((depth % 4) * 2))) & 0b11) as usize
        ((key[depth / 8] >> (7 - (depth % 8))) & 0b1) as usize
    }

    /*const*/ fn subtract_arrays(array_1: &[u8;N], array_2: &[u8;N]) -> [u8;N]{
        let mut result = [0; N];

        let mut borrow = false;

        for i in (0..N).rev(){
            if array_1[i] > array_2[i] {
                result[i] = array_1[i] - array_2[i];
                if borrow{
                    result[i] -= 1;
                    borrow = false;
                }
            }
            else if array_1[i] == array_2[i] {result[i] = if borrow {u8::MAX} else {0}}
            else{
                //smaller[i] must be larger than bigger[i]
                //in case where smaller[i] = 255 and larger[i] = 0
                //left side is (255 - 255) + 0 + 1 = 1
                //this is lowest value can be so subtract 1 from this will never cause error
                result[i] = (u8::MAX - array_2[i]) + array_1[i];
                if !borrow {
                    result[i] += 1;
                    borrow = true;
                }
            }
        }

        result
    }

    /*const*/ fn get_abs_diff(key_1: &[u8;N], key_2: &[u8;N]) -> [u8;N]{
        Self::subtract_arrays(key_1, key_2).min(Self::subtract_arrays(key_2, key_1))
    }

    pub const fn new() -> Self {Self::Leaf(None)}

    /*const*/ fn _contains(&self, key: &[u8;N], depth: usize) -> bool{
        match self{
            Self::Branch(branch) => branch[Self::get_index(key, depth)]._contains(key, depth + 1),
            Self::Leaf(None) => false,
            Self::Leaf(Some((k, _))) => k == key
        }
    }

    pub /*const*/ fn contains(&self, key: &[u8; N]) -> bool {self._contains(key, 0)}

    pub /*const*/ fn entries(&self, exclude: &ByteArrayTreeSet<N>) -> u64{
        match self{
            Self::Branch(branch) => branch.iter().map(|x| x.entries(exclude)).sum(),
            Self::Leaf(None) => 0,
            Self::Leaf(Some(_)) => 1
        }
    }

    pub /*const*/ fn get_min_key(&self, exclude: &ByteArrayTreeSet<N>) -> Option<[u8; N]> {
        match self{
            Self::Branch(branch) => branch.iter().find_map(|x| x.get_min_key(exclude)),
            Self::Leaf(leaf) => leaf.as_ref().filter(|(k,_)| !exclude.contains(k)).map(|(k,_)| *k)
        }
    }

    pub /*const*/ fn get_max_key(&self, exclude: &ByteArrayTreeSet<N>) -> Option<[u8; N]> {
        match self{
            Self::Branch(branch) => branch.iter().rev().find_map(|x| x.get_max_key(exclude)),
            Self::Leaf(leaf) => leaf.as_ref().filter(|(k,_)| !exclude.contains(k)).map(|(k,_)| *k)
        }
    }

    pub /*const*/ fn max_depth(&self) -> u64 {
        match self{
            Self::Branch(branch) => 1 + branch.iter().map(|x| x.max_depth()).max().unwrap_or_default(),
            Self::Leaf(_) => 0,
        }
    }

    /*const*/ fn _highest_shared_leading_zeroes_by_key(&self, key: &[u8; N], exclude: &ByteArrayTreeSet<N>, depth: usize) -> Option<[u8; N]> {
        match self{
            Self::Branch(branch) => {
                //const KEY_ORDER: [[usize; 4]; 4] = [[0,1,2,3], [1,0,2,3], [2,3,0,1], [3,2,0,1]];
                const KEY_ORDER: [[usize; 2]; 2] = [[0,1], [1,0]];
                KEY_ORDER[Self::get_index(key, depth)].into_iter()
                    .find_map(|i| branch[i]._highest_shared_leading_zeroes_by_key(key, exclude, depth + 1))
            }
            Self::Leaf(leaf) => leaf.as_ref().filter(|(k,_)| (k != key) && !exclude.contains(k)).map(|(k,_)| *k)
        }
    }

    pub /*const*/ fn highest_shared_leading_zeroes_by_key(&self, key: &[u8; N], exclude: &ByteArrayTreeSet<N>) -> Option<[u8; N]> {
        self._highest_shared_leading_zeroes_by_key(key, exclude, 0)
    }

    /*const*/ fn _closest_by_abs_distance_by_key(&self, key: &[u8; N], exclude: &ByteArrayTreeSet<N>, depth: usize) -> (Option<([u8;N], [u8;N])>, bool, bool) {
        match self{
            Self::Branch(branch) => {
                let index = Self::get_index(key, depth);

                let (mut closest, mut left, mut right) = branch[index]._closest_by_abs_distance_by_key(key, exclude, depth + 1);

                let mut insert_closest = |value|{
                    let abs_diff = Self::get_abs_diff(&value, key);
                    if closest.is_none_or(|(_, x)| abs_diff < x) { closest = Some((value, abs_diff)) }
                };

                if !left{
                    match if index == 0 {if depth == 0 /*must be smallest entry*/ {self.get_max_key(exclude)} else {None}}
                    else {branch[1].get_max_key(exclude)}
                    {
                        Some(value) => {
                            insert_closest(value);
                            left = true;
                        }
                        None => ()
                    }
                }

                if !right{
                    match if index == 1 {if depth == 0 /*must be largest entry*/ {self.get_min_key(exclude)} else {None}}
                    else {branch[0].get_min_key(exclude)}
                    {
                        Some(value) => {
                            insert_closest(value);
                            right = true;
                        }
                        None => ()
                    }
                }

                (closest, left, right)
            }
            Self::Leaf(leaf) => (
                leaf.as_ref().filter(|(k,_)| (k != key) && !exclude.contains(k)).map(|(k,_)| (*k, Self::get_abs_diff(k, key))),
                false,
                false
            )
        }
    }

    pub /*const*/ fn closest_by_abs_distance_by_key(&self, key: &[u8; N], exclude: &ByteArrayTreeSet<N>) -> Option<[u8; N]> {
        self._closest_by_abs_distance_by_key(key, exclude, 0).0.map(|(k,_)| k)
    }

    /*const*/ fn _get(&self, key: &[u8;N], depth: usize) -> Option<&V>{
        match self{
            Self::Branch(branch) => branch[Self::get_index(key, depth)]._get(key, depth + 1),
            Self::Leaf(None) => None,
            Self::Leaf(Some((k,v))) => if k == key {Some(v)} else {None}
        }
    }

    pub /*const*/ fn get(&self, key: &[u8; N]) -> Option<&V> {self._get(key, 0)}

    pub fn clear(&mut self) {*self = Self::new()}
}

//try to remove requirement for V to be clonable for insert and remove

impl<const N: usize, V: Clone> ByteArrayTreeMap<N, V>{
    fn _insert_or_update_if(&mut self, key: &[u8; N], value: V, condition: &impl Fn(&V) -> bool, depth: usize) -> bool {
        match self{
            Self::Branch(branch) => branch[Self::get_index(key, depth)]._insert_or_update_if(key, value, condition, depth + 1),
            Self::Leaf(leaf) => {
                match leaf{
                    None => *leaf = Some((*key, value)), //insert
                    Some((k, v)) => {
                        if k == key {
                            if !condition(v) {return false}
                            //update
                            *v = value
                        }
                        else{
                            //insert
                            *self = {
                                let mut new_branch = Self::Branch(core::array::from_fn(|_| Box::new(Self::new())));
                                new_branch._insert_or_update_if(k, v.clone(), condition, depth);
                                new_branch._insert_or_update_if(key, value, condition, depth);
                                new_branch
                            }
                        }
                    }
                }
                true
            }
        }
    }

    pub fn insert_or_update_if(&mut self, key: &[u8; N], value: V, condition: &impl Fn(&V) -> bool) -> bool {
        self._insert_or_update_if(key, value, condition, 0)
    }

    pub fn insert(&mut self, key: &[u8; N], value: V) {
        _ = self.insert_or_update_if(key, value, &|_| true)
    }

    fn _remove_if(&mut self, key: &[u8; N], condition: &impl Fn(&V) -> bool, depth: usize) -> bool{
        match self{
            Self::Branch(branch) => {
                if !branch[Self::get_index(key, depth)]._remove_if(key, condition, depth + 1) {return false}
                let mut trim = Some(false);
                for entry in branch.iter(){
                    match entry.as_ref(){
                        Self::Leaf(None) => (),
                        Self::Leaf(Some(_)) => trim = if trim == Some(false) {Some(true)} else {None},
                        Self::Branch(_) => trim = None
                    }
                    if trim.is_none() {break}
                }
                match trim{
                    Some(false) => *self = Self::new(),
                    Some(true) => {
                        for entry in branch.iter(){
                            match entry.as_ref(){
                                Self::Leaf(Some(leaf)) => {
                                    *self = Self::Leaf(Some(leaf.clone()));
                                    break;
                                },
                                _ => ()
                            }
                        }
                    }
                    None => ()
                }
                true
            }
            Self::Leaf(leaf) => {
                match leaf{
                    Some((k,v)) => {
                        if (k == key) && condition(v){
                            *leaf = None;
                            true
                        }
                        else {false}
                    }
                    None => false
                }
            }
        }
    }

    pub fn remove_if(&mut self, key: &[u8; N], condition: &impl Fn(&V) -> bool) -> bool {
        self._remove_if(key, condition, 0)
    }

    pub fn remove(&mut self, key: &[u8; N]) -> bool {
        self.remove_if(key, &|_| true)
    }
}

pub struct ByteArrayTreeSet<const N: usize>(ByteArrayTreeMap<N, ()>);

impl<const N: usize> ByteArrayTreeSet<N>{
    pub const fn new() -> Self {Self(ByteArrayTreeMap::new())}

    pub /*const*/ fn contains(&self, key: &[u8; N]) -> bool {self.0.contains(key)}

    pub /*const*/ fn entries(&self, exclude: &Self) -> u64 {self.0.entries(exclude)}

    pub /*const*/ fn get_min_key(&self, exclude: &Self) -> Option<[u8; N]> {self.0.get_min_key(exclude)}

    pub /*const*/ fn get_max_key(&self, exclude: &Self) -> Option<[u8; N]> {self.0.get_max_key(exclude)}

    pub /*const*/ fn max_depth(&self) -> u64 {self.0.max_depth()}

    pub /*const*/ fn highest_shared_leading_zeroes_by_key(&self, key: &[u8; N], exclude: &Self) -> Option<[u8; N]>{
        self.0.highest_shared_leading_zeroes_by_key(key, exclude)
    }

    pub /*const*/ fn closest_by_abs_distance_by_key(&self, key: &[u8; N], exclude: &Self) -> Option<[u8; N]>{
        self.0.closest_by_abs_distance_by_key(key, exclude)
    }

    fn clear(&mut self) {self.0.clear()}

    fn insert(&mut self, key: &[u8; N]) {self.0.insert(key, ())}

    fn remove(&mut self, key: &[u8; N]) -> bool {self.0.remove(key)}
}