use slotmap::{DefaultKey, SlotMap};

struct Node<T> {
    prev: Option<DefaultKey>,
    next: Option<DefaultKey>,
    val: T,
}

#[derive(Default)]
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
    pub fn push(&mut self, val: T) -> DefaultKey {
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

    pub fn cursor_from_id(&self, id: DefaultKey) -> Option<Cursor> {
        if let Some(node) = self.slots.get(id) {
            Some(Cursor::from_node(node, id))
        } else {
            None
        }
    }

    pub fn head(&self) -> Option<Cursor> {
        if let Some(HeadTail { head, .. }) = self.head_tail {
            let node = self.slots.get(head).unwrap();
            Some(Cursor::from_node(node, head))
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: DefaultKey) -> Option<T> {
        let (prev, next, ret) = if let Some(victim) = self.slots.remove(id) {
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
}

#[derive(Copy, Clone, Debug)]
pub struct Cursor {
    id: DefaultKey,
    next: Option<DefaultKey>,
    prev: Option<DefaultKey>,
}

impl Cursor {
    fn from_node<T>(node: &Node<T>, id: DefaultKey) -> Self {
        Self {
            id,
            prev: node.prev,
            next: node.next,
        }
    }

    pub fn get<'a, T>(&self, map: &'a LinkedSlotlist<T>) -> Option<&'a T> {
        if let Some(node) = map.slots.get(self.id) {
            Some(&node.val)
        } else {
            None
        }
    }

    pub fn id(&self) -> DefaultKey {
        self.id
    }

    pub fn next(&self) -> Option<DefaultKey> {
        self.next
    }

    pub fn prev(&self) -> Option<DefaultKey> {
        self.prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn collect_forward(mut cursor: Cursor, slotmap: &LinkedSlotlist<u32>) -> (Cursor, Vec<u32>) {
        let mut ret = Vec::new();
        while let Some(next) = cursor.next() {
            ret.push(*cursor.get(&slotmap).unwrap());
            cursor = slotmap.cursor_from_id(next).unwrap();
        }
        ret.push(*cursor.get(&slotmap).unwrap());
        (cursor, ret)
    }

    fn collect_backward(mut cursor: Cursor, slotmap: &LinkedSlotlist<u32>) -> (Cursor, Vec<u32>) {
        let mut ret = Vec::new();
        while let Some(prev) = cursor.prev() {
            ret.push(*cursor.get(&slotmap).unwrap());
            cursor = slotmap.cursor_from_id(prev).unwrap();
        }
        ret.push(*cursor.get(&slotmap).unwrap());
        (cursor, ret)
    }

    #[test]
    fn just_do_something_and_hope_it_works() {
        let mut slotmap = LinkedSlotlist::default();
        let one = slotmap.push(1);
        slotmap.push(2);
        slotmap.push(3);
        slotmap.push(4);

        let cursor = slotmap.head().unwrap();

        let (cursor, ret) = collect_forward(cursor, &slotmap);
        assert_eq!(ret, vec![1, 2, 3, 4]);

        let (_, ret) = collect_backward(cursor, &slotmap);
        assert_eq!(ret, vec![4, 3, 2, 1]);

        slotmap.remove(one);
        let cursor = slotmap.head().unwrap();
        let (_, ret) = collect_forward(cursor, &slotmap);
        assert_eq!(ret, vec![2, 3, 4]);

        slotmap.push(5);
        let cursor = slotmap.head().unwrap();
        let (_, ret) = collect_forward(cursor, &slotmap);
        assert_eq!(ret, vec![2, 3, 4, 5]);
    }
}
