use std::collections::LinkedList;

#[derive(Debug)]
pub struct ConstrainedSortedList<T>
where
    T: PartialOrd + Clone,
{
    size: usize,
    capacity: usize,
    list: LinkedList<T>,
}

impl<T> ConstrainedSortedList<T>
where
    T: PartialOrd + Clone,
{
    pub fn new(capacity: usize) -> ConstrainedSortedList<T> {
        ConstrainedSortedList {
            capacity,
            size: 0,
            list: LinkedList::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.list.iter()
    }

    pub fn insert_maybe(&mut self, el: &T) {
        let mut cursor = self.list.cursor_front_mut();

        for _ in 0..self.capacity {
            match cursor.current() {
                Some(node) => {
                    if *node >= *el {
                        cursor.insert_before(el.clone());
                        if self.size == self.capacity {
                            self.list.pop_back();
                        } else {
                            self.size += 1;
                        }
                        break;
                    }
                }
                None => {
                    cursor.insert_before(el.clone());
                    self.size += 1;
                    break;
                }
            }
            cursor.move_next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut csl = ConstrainedSortedList::new(3);
        csl.insert_maybe(&(1 as usize));
        csl.insert_maybe(&(5 as usize));
        csl.insert_maybe(&(3 as usize));
        csl.insert_maybe(&(4 as usize));
        csl.insert_maybe(&(2 as usize));
        csl.insert_maybe(&(6 as usize));

        let mut iter = csl.iter();

        assert_eq!(Some(&6), iter.next());
        assert_eq!(Some(&5), iter.next());
        assert_eq!(Some(&4), iter.next());
        assert_eq!(None, iter.next());
    }
}
