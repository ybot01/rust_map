#![no_std]
extern crate alloc;

use alloc::boxed::Box;

const fn get_branch_index<const N: usize>(key: &[u8;N], depth: usize) -> usize {
    ((key[depth / 8] >> (7 - (depth % 8))) & 0b1) as usize
}

/*const*/ fn subtract_arrays<const N: usize>(array_1: &[u8;N], array_2: &[u8;N]) -> [u8;N]{
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

/*const*/ fn get_abs_diff<const N: usize>(key_1: &[u8;N], key_2: &[u8;N]) -> [u8;N]{
    subtract_arrays(key_1, key_2).min(subtract_arrays(key_2, key_1))
}

#[derive(Debug, PartialEq, Eq)]
pub enum ByteArrayTreeMap<const N: usize, V>{
    Branch([Option<Box<Self>>; 2]),
    Leaf([u8;N], V)
}

impl<const N: usize, V> ByteArrayTreeMap<N, V>{

    pub const fn new() -> Self {Self::Branch([const {None}; 2])}

    pub /*const*/ fn is_empty(&self) -> bool{
        match self{
            Self::Branch(branch) => branch.iter().all(|x| x.is_none()),
            Self::Leaf(..) => false
        }
    }

    /*const*/ fn _contains(&self, key: &[u8;N], depth: usize) -> bool{
        match self{
            Self::Branch(branch) => branch[get_branch_index(key, depth)].as_ref()
                .is_some_and(|entry| entry._contains(key, depth + 1)),
            Self::Leaf(k,_) => k == key
        }
    }

    pub /*const*/ fn contains(&self, key: &[u8; N]) -> bool {self._contains(key, 0)}

    pub /*const*/ fn entries(&self, exclude: &ByteArrayTreeSet<N>) -> u64{
        match self{
            Self::Branch(branch) => branch.iter()
                .map(|entry| entry.as_ref())
                .filter_map(|entry| entry)
                .map(|entry| entry.entries(exclude))
                .sum(),
            Self::Leaf(..) => 1
        }
    }

    pub /*const*/ fn get_min_key(&self, exclude: &ByteArrayTreeSet<N>) -> Option<[u8; N]> {
        match self{
            Self::Branch(branch) => {
                branch.iter()
                    .map(|entry| entry.as_ref())
                    .filter_map(|entry| entry)
                    .find_map(|entry| entry.get_min_key(exclude))
            },
            Self::Leaf(k, _) => if !exclude.contains(k) {Some(*k)} else {None}
        }
    }

    pub /*const*/ fn get_max_key(&self, exclude: &ByteArrayTreeSet<N>) -> Option<[u8; N]> {
        match self{
            Self::Branch(branch) => {
                branch.iter().rev()
                    .map(|x| x.as_ref())
                    .filter_map(|entry| entry)
                    .find_map(|entry| entry.get_max_key(exclude))
            },
            Self::Leaf(k, _) => if !exclude.contains(k) {Some(*k)} else {None}
        }
    }

    pub /*const*/ fn max_depth(&self) -> u64 {
        match self{
            Self::Branch(branch) => 1 + branch.iter()
                .map(|entry| entry.as_ref())
                .filter_map(|entry| entry)
                .map(|x| x.max_depth())
                .max()
                .unwrap_or(0),
            Self::Leaf(..) => 1
        }
    }

    /*const*/ fn _highest_shared_leading_zeroes_by_key(&self, key: &[u8; N], exclude: &ByteArrayTreeSet<N>, depth: usize) -> Option<[u8; N]> {
        match self{
            Self::Branch(branch) => {
                //const KEY_ORDER: [[usize; 4]; 4] = [[0,1,2,3], [1,0,2,3], [2,3,0,1], [3,2,0,1]];
                const BRANCH_INDEX_ORDER: [[usize; 2]; 2] = [[0,1], [1,0]];

                BRANCH_INDEX_ORDER[get_branch_index(key, depth)].into_iter().find_map(|branch_index|
                    branch[branch_index].as_ref()
                        .and_then(|found| found._highest_shared_leading_zeroes_by_key(key, exclude, depth + 1))
                )
            }
            Self::Leaf(k, _) => if (k != key) && !exclude.contains(k) {Some(*k)} else {None}
        }
    }

    pub /*const*/ fn highest_shared_leading_zeroes_by_key(&self, key: &[u8; N], exclude: &ByteArrayTreeSet<N>) -> Option<[u8; N]> {
        self._highest_shared_leading_zeroes_by_key(key, exclude, 0)
    }

    /*const*/ fn _closest_by_abs_distance_by_key(&self, key: &[u8; N], exclude: &ByteArrayTreeSet<N>, depth: usize) -> (Option<([u8;N], [u8;N])>, bool, bool) {
        match self{
            Self::Branch(branch) => {
                let branch_index = get_branch_index(key, depth);

                let (mut closest, mut left, mut right) = branch[branch_index].as_ref()
                    .map_or(
                        (None, false, false),
                        |found| found._closest_by_abs_distance_by_key(key, exclude, depth + 1)
                    );

                let mut insert_closest = |value|{
                    let abs_diff = get_abs_diff(&value, key);
                    if closest.is_none_or(|(_, x)| abs_diff < x) { closest = Some((value, abs_diff)) }
                };

                if !left{
                    match if branch_index == 0 {if depth == 0 /*must be smallest entry*/ {self.get_max_key(exclude)} else {None}}
                    else {branch[0].as_ref().and_then(|found| found.get_max_key(exclude))}
                    {
                        Some(value) => {
                            insert_closest(value);
                            left = true;
                        }
                        None => ()
                    }
                }

                if !right{
                    match if branch_index == 1 {if depth == 0 /*must be largest entry*/ {self.get_min_key(exclude)} else {None}}
                    else {branch[1].as_ref().and_then(|found| found.get_min_key(exclude))}
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
            Self::Leaf(k,_) => (
                if (k != key) && !exclude.contains(k) {Some((*k, get_abs_diff(k, key)))} else {None},
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
            Self::Branch(branch) => branch[get_branch_index(key, depth)].as_ref()
                .and_then(|found| found._get(key, depth + 1)),
            Self::Leaf(k,v) => if k == key {Some(v)} else {None}
        }
    }

    pub /*const*/ fn get(&self, key: &[u8; N]) -> Option<&V> {self._get(key, 0)}

    pub fn clear(&mut self) {*self = Self::new()}

    fn _insert_or_update_if_mut(&mut self, key: &[u8; N], value: V, condition: &impl Fn(&mut V) -> bool, depth: usize) -> bool {
        match self{
            Self::Branch(branch) => {
                let branch_index = get_branch_index(key, depth);
                match branch[branch_index].as_mut(){
                    Some(found) => found._insert_or_update_if_mut(key, value, condition, depth + 1),
                    None => {
                        branch[branch_index] = Some(Box::new(Self::Leaf(*key, value)));
                        true
                    }
                }
            },
            Self::Leaf(k, v) => {
                if k == key {
                    if !condition(v) {return false}
                    //update
                    *v = value
                }
                else{
                    //insert
                    match core::mem::replace(self, Self::new()) {
                        Self::Leaf(old_k, old_v) => {
                            //passing condition doesnt matter here
                            //there are no existing keys in self so inserting existing key will insert
                            //then inserting new key we know is not same as existing key so will insert
                            self._insert_or_update_if_mut(&old_k, old_v, condition, depth);
                            self._insert_or_update_if_mut(key, value, condition, depth);
                        }
                        Self::Branch(_) => unreachable!()
                    }
                }
                true
            }
        }
    }

    pub fn insert_or_update_if(&mut self, key: &[u8; N], value: V, condition: &impl Fn(&V) -> bool) -> bool {
        self._insert_or_update_if_mut(key, value, &|v| condition(v), 0)
    }

    pub fn insert_or_update_if_mut(&mut self, key: &[u8; N], value: V, condition: &impl Fn(&mut V) -> bool) -> bool {
        self._insert_or_update_if_mut(key, value, condition, 0)
    }

    pub fn insert(&mut self, key: &[u8; N], value: V) {
        _ = self.insert_or_update_if(key, value, &|_| true)
    }

    fn _remove_if_mut(&mut self, key: &[u8; N], condition: &impl Fn(&mut V) -> bool, depth: usize) -> bool{
        match self{
            Self::Branch(branch) => {
                let branch_index = get_branch_index(key, depth);
                if !branch[branch_index].as_mut()
                    .map_or(
                        false,
                        |found| found._remove_if_mut(key, condition, depth + 1)
                    )
                {
                    return false;
                }

                //check if current branch has empty branch inside, set these to none

                if branch[branch_index].as_ref().is_some_and(|entry| entry.is_empty()) {branch[branch_index] = None}

                let mut leaf_index = Some(None);

                for i in 0..branch.len() {
                    match branch[i].as_ref().map(|x| x.as_ref()){
                        Some(Self::Branch(_)) => leaf_index = None, //exit, dont prune
                        Some(Self::Leaf(..)) => leaf_index = if leaf_index == Some(None) {Some(Some(i))} else {None}, //if multiple leafs exit, dont prune
                        None => ()
                    }
                    if leaf_index.is_none() {break}
                }

                //if branch contains only 1 leaf and 1 none entry then bring the leaf entry up a level
                match leaf_index{
                    Some(Some(index)) => *self = *branch[index].take().unwrap(),
                    _ => ()
                }

                true
            }
            Self::Leaf(k, v) => {
                if (k == key) && condition(v){
                    *self = Self::new();
                    true
                }
                else {false}
            }
        }
    }

    pub fn remove_if(&mut self, key: &[u8; N], condition: &impl Fn(&V) -> bool) -> bool {
        self._remove_if_mut(key, &|v| condition(v), 0)
    }

    pub fn remove_if_mut(&mut self, key: &[u8; N], condition: &impl Fn(&mut V) -> bool) -> bool {
        self._remove_if_mut(key, condition, 0)
    }

    pub fn remove(&mut self, key: &[u8; N]) -> bool {
        self.remove_if(key, &|_| true)
    }
}

pub struct ByteArrayTreeSet<const N: usize>(ByteArrayTreeMap<N, ()>);

impl<const N: usize> ByteArrayTreeSet<N>{
    pub const fn new() -> Self {Self(ByteArrayTreeMap::new())}
    
    pub /*const*/ fn is_empty(&self) -> bool {self.0.is_empty()}

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