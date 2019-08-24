pub use slotmap::DefaultKey;
use slotmap::SlotMap;
// TODO: better API

struct Node<T> {
    prev: Option<DefaultKey>,
    next: Option<DefaultKey>,
    val: T,
}

pub struct LinkedSlotlist<T> {
    slots: SlotMap<DefaultKey, Node<T>>,
    head_tail: Option<HeadTail>,
}

#[derive(Clone, Copy)]
struct HeadTail {
    head: DefaultKey,
    tail: DefaultKey,
}

impl<T> LinkedSlotlist<T> {
    pub fn new() -> Self {
        LinkedSlotlist {
            slots: SlotMap::new(),
            head_tail: None,
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        LinkedSlotlist {
            slots: SlotMap::with_capacity(n),
            head_tail: None,
        }
    }

    pub fn push_back(&mut self, val: T) -> DefaultKey {
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
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let HeadTail { head, .. } = self.head_tail?;

        self.remove(head)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let HeadTail { tail, .. } = self.head_tail?;

        self.remove(tail)
    }

    pub fn head(&self) -> Option<DefaultKey> {
        self.head_tail.map(|HeadTail { head, .. }| head)
    }

    pub fn tail(&self) -> Option<DefaultKey> {
        self.head_tail.map(|HeadTail { tail, .. }| tail)
    }

    pub fn remove(&mut self, victim: DefaultKey) -> Option<T> {
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

    pub fn get(&self, key: DefaultKey) -> Option<&T> {
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

    pub fn next(&self, id: DefaultKey) -> Option<DefaultKey> {
        let node = self.slots.get(id)?;
        node.next
    }

    pub fn prev(&self, id: DefaultKey) -> Option<DefaultKey> {
        let node = self.slots.get(id)?;
        node.prev
    }
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

    #[test]
    fn just_do_something_and_hope_it_works() {
        let mut slotmap = LinkedSlotlist::new();
        let one = slotmap.push_back(1);
        slotmap.push_back(2);
        slotmap.push_back(3);
        slotmap.push_back(4);

        let cursor = slotmap.head().unwrap();

        let (cursor, ret) = collect_forward(cursor, &slotmap);
        assert_eq!(ret, vec![1, 2, 3, 4]);

        let (_, ret) = collect_backward(cursor, &slotmap);
        assert_eq!(ret, vec![4, 3, 2, 1]);

        slotmap.remove(one);
        let cursor = slotmap.head().unwrap();
        let (_, ret) = collect_forward(cursor, &slotmap);
        assert_eq!(ret, vec![2, 3, 4]);

        slotmap.push_back(5);
        let cursor = slotmap.head().unwrap();
        let (_, ret) = collect_forward(cursor, &slotmap);
        assert_eq!(ret, vec![2, 3, 4, 5]);
    }
}
