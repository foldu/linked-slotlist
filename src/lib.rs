pub use slotmap::DefaultKey;
use slotmap::{Key, SlotMap};
// TODO: better API

struct Node<T, K>
where
    K: Key,
{
    prev: Option<K>,
    next: Option<K>,
    val: T,
}

pub struct LinkedSlotlist<T, K = DefaultKey>
where
    K: Key,
{
    slots: SlotMap<K, Node<T, K>>,
    head_tail: Option<HeadTail<K>>,
}

#[derive(Clone, Copy)]
struct HeadTail<K>
where
    K: Key,
{
    head: K,
    tail: K,
}

impl<T, K> LinkedSlotlist<T, K>
where
    K: Key + Copy,
{
    pub fn new() -> Self {
        LinkedSlotlist {
            slots: SlotMap::with_key(),
            head_tail: None,
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        LinkedSlotlist {
            slots: SlotMap::with_capacity_and_key(n),
            head_tail: None,
        }
    }

    pub fn insert_before(&mut self, key: K, val: T) -> Option<K> {
        let next_prev = self.slots.get(key).map(|node| node.prev)?;
        match next_prev {
            // last element, just call push_front to handle the gnarly stuff
            None => Some(self.push_front(val)),
            Some(prev) => {
                let node = Node {
                    prev: Some(prev),
                    next: Some(key),
                    val,
                };
                let ret = self.slots.insert(node);
                let prev_node = self.slots.get_mut(prev)?;

                prev_node.next = Some(ret);
                if let Some(next) = self.slots.get_mut(key) {
                    next.prev = Some(ret);
                }

                Some(ret)
            }
        }
    }

    pub fn insert_after(&mut self, key: K, val: T) -> Option<K> {
        let prev_next = self.slots.get(key).map(|node| node.next)?;
        match prev_next {
            // last element, just call push_back to handle the gnarly stuff
            None => Some(self.push_back(val)),
            Some(next) => {
                let node = Node {
                    prev: Some(key),
                    next: Some(next),
                    val,
                };
                let ret = self.slots.insert(node);
                let next_node = self.slots.get_mut(next)?;

                next_node.prev = Some(ret);
                if let Some(prev) = self.slots.get_mut(key) {
                    prev.next = Some(ret);
                }

                Some(ret)
            }
        }
    }

    pub fn push_front(&mut self, val: T) -> K {
        if let Some(HeadTail { ref mut head, .. }) = self.head_tail {
            let this = self.slots.insert(Node {
                prev: None,
                next: Some(*head),
                val,
            });
            let mut prev_head = self.slots.get_mut(*head).unwrap();
            prev_head.prev = Some(this);
            *head = this;
            this
        } else {
            self.insert_first_elem(val)
        }
    }

    pub fn push_back(&mut self, val: T) -> K {
        if let Some(HeadTail { ref mut tail, .. }) = self.head_tail {
            let this = self.slots.insert(Node {
                prev: Some(*tail),
                next: None,
                val,
            });
            let mut prev_last = self.slots.get_mut(*tail).unwrap();
            prev_last.next = Some(this);
            *tail = this;
            this
        } else {
            self.insert_first_elem(val)
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let HeadTail { head, .. } = self.head_tail?;

        self.remove(head)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let HeadTail { tail, .. } = self.head_tail?;

        self.remove(tail)
    }

    pub fn head(&self) -> Option<K> {
        self.head_tail.map(|HeadTail { head, .. }| head)
    }

    pub fn tail(&self) -> Option<K> {
        self.head_tail.map(|HeadTail { tail, .. }| tail)
    }

    pub fn remove(&mut self, victim: K) -> Option<T> {
        let (prev, next, ret) = if let Some(victim) = self.slots.remove(victim) {
            (victim.prev, victim.next, victim.val)
        } else {
            return None;
        };

        let head_tail = self.head_tail.unwrap();
        self.head_tail = match (prev, next) {
            // victim was the only element
            (None, None) => None,
            // victim was head
            (None, _) => Some(HeadTail {
                head: next.unwrap(),
                ..head_tail
            }),
            // victim was tail
            (_, None) => Some(HeadTail {
                tail: prev.unwrap(),
                ..head_tail
            }),
            // nothing interesting
            _ => Some(head_tail),
        };

        if let Some(node) = prev.and_then(|prev| self.slots.get_mut(prev)) {
            node.next = next;
        }

        if let Some(node) = next.and_then(|next| self.slots.get_mut(next)) {
            node.prev = prev;
        }

        Some(ret)
    }

    pub fn get(&self, key: K) -> Option<&T> {
        self.slots.get(key).map(|node| &node.val)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slots.values().map(|v| &v.val)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.values_mut().map(|v| &mut v.val)
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn next(&self, id: K) -> Option<K> {
        let node = self.slots.get(id)?;
        node.next
    }

    pub fn prev(&self, id: K) -> Option<K> {
        let node = self.slots.get(id)?;
        node.prev
    }
}

impl<T, K> LinkedSlotlist<T, K>
where
    K: Key + Copy,
{
    fn insert_first_elem(&mut self, val: T) -> K {
        let this = self.slots.insert(Node {
            prev: None,
            next: None,
            val,
        });

        self.head_tail = Some(HeadTail {
            head: this,
            tail: this,
        });
        this
    }

    // probably going to need this for append
    //fn contiguous_list<I>(&mut self, it: I) -> Option<HeadTail<K>>
    //where
    //    I: IntoIterator<Item = T>,
    //{
    //    let mut it = it.into_iter();

    //    let first = it.next()?;

    //    let key = self.slots.insert(Node {
    //        val: first,
    //        prev: None,
    //        next: None,
    //    });

    //    let mut ret = HeadTail {
    //        head: key,
    //        tail: key,
    //    };

    //    for val in it {
    //        let node = self.slots.insert(Node {
    //            val,
    //            prev: Some(ret.tail),
    //            next: None,
    //        });

    //        self.slots.get_mut(ret.tail).unwrap().next = Some(node);
    //        ret.tail = node;
    //    }

    //    Some(ret)
    //}
}

impl<T> std::iter::FromIterator<T> for LinkedSlotlist<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let mut ret = if let (_, Some(len)) = iter.size_hint() {
            LinkedSlotlist::with_capacity(len)
        } else {
            LinkedSlotlist::new()
        };

        for it in iter {
            ret.push_back(it);
        }

        ret
    }
}

impl<T> Default for LinkedSlotlist<T> {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: more tests
#[cfg(test)]
mod tests {
    use super::*;
    fn collect_forward(
        mut cursor: DefaultKey,
        list: &LinkedSlotlist<u32>,
    ) -> (DefaultKey, Vec<u32>) {
        let mut ret = Vec::new();
        while let Some(next) = list.next(cursor) {
            ret.push(*list.get(cursor).unwrap());
            cursor = next;
        }
        ret.push(*list.get(cursor).unwrap());
        (cursor, ret)
    }

    fn collect_backward(
        mut cursor: DefaultKey,
        list: &LinkedSlotlist<u32>,
    ) -> (DefaultKey, Vec<u32>) {
        let mut ret = Vec::new();
        while let Some(prev) = list.prev(cursor) {
            ret.push(*list.get(cursor).unwrap());
            cursor = prev;
        }
        ret.push(*list.get(cursor).unwrap());
        (cursor, ret)
    }

    fn collect_entire_list(list: &LinkedSlotlist<u32>) -> Vec<u32> {
        if let Some(head) = list.head() {
            let (_, ret) = collect_forward(head, list);
            ret
        } else {
            vec![]
        }
    }

    #[test]
    fn just_do_something_and_hope_it_works() {
        let mut list = LinkedSlotlist::new();
        let one = list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.push_back(4);

        let cursor = list.head().unwrap();

        let (cursor, ret) = collect_forward(cursor, &list);
        assert_eq!(ret, vec![1, 2, 3, 4]);

        let (_, ret) = collect_backward(cursor, &list);
        assert_eq!(ret, vec![4, 3, 2, 1]);

        list.remove(one);
        let cursor = list.head().unwrap();
        let (_, ret) = collect_forward(cursor, &list);
        assert_eq!(ret, vec![2, 3, 4]);

        list.push_back(5);
        let cursor = list.head().unwrap();
        let (_, ret) = collect_forward(cursor, &list);
        assert_eq!(ret, vec![2, 3, 4, 5]);

        list.push_front(1);
        let ret = collect_entire_list(&list);
        assert_eq!(ret, vec![1, 2, 3, 4, 5]);

        let cursor = list.head().unwrap();
        let cursor = list.next(cursor).unwrap();
        list.insert_after(cursor, 3);
        let ret = collect_entire_list(&list);
        assert_eq!(ret, vec![1, 2, 3, 3, 4, 5]);
    }
}
