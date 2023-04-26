use std::collections::LinkedList;

// prepend(a, b) moves all the elements of a to the beginning of b, emptying a in
//  the process
//
// Runs in O(1)
//
// TODO (low priority) at some point it might be good to check how exactly the 
//  LinkedList ADT is structured in Rust and what append is doing. It says O(1),
//  but is it using pointers effectively?
pub fn prepend<T>(a: &mut LinkedList<T>, b: &mut LinkedList<T>) {
    a.append(b);
    b.append(a);
}

// insert_just_before(ll, items, condition) finds the first element
//  of `ll` which satisfies `condition` and inserts all the elements of `items`
//  just before it. After this operation, `items` will empty.
//
// This function runs in O(n) time where n is the length of `ll` so long as 
//  condition is O(1)
pub fn insert_just_before<T> (ll: &mut LinkedList<T>, items: &mut LinkedList<T>, condition: impl Fn(&T) -> bool) {
    let mut checked: LinkedList<T> = LinkedList::new();
    loop {
        match ll.pop_front() {

            Some(t) => if condition(&t) {
                    ll.push_front(t);
                    break;
                } else {
                    checked.push_back(t);
                    continue;
                },

            None => break,
        };
    }

    checked.append(items);
    prepend(&mut checked,  ll);
}

#[test]
fn test1() {
    let mut ll = LinkedList::from([1,2,3,4,5,4,3,2,1,0]);
    insert_just_before(&mut ll, &mut LinkedList::from([0]), |x| *x == 5);
    assert_eq!(ll, LinkedList::from([1,2,3,4,0,5,4,3,2,1,0]));
}

#[cfg(test)] 
fn even(x: &i32) -> bool {
    return x%2 == 0;
}

#[test]
fn test2() {
    let mut ll = LinkedList::from([1,2,3,4,5,4,3,2,1,0]);
    insert_just_before(&mut ll, &mut LinkedList::from([0]), &even);
    assert_eq!(ll, LinkedList::from([1,0,2,3,4,5,4,3,2,1,0]));
}

#[test]
fn test3() {
    let mut ll = LinkedList::from([]);
    insert_just_before(&mut ll, &mut LinkedList::from([0]), &even);
    assert_eq!(ll, LinkedList::from([0]));
}

#[test]
fn test4() {
    let mut ll = LinkedList::from([1,3,5,7]);
    insert_just_before(&mut ll, &mut LinkedList::from([0]), &even);
    assert_eq!(ll, LinkedList::from([1,3,5,7,0]));
}

#[test]
fn test5() {
    let mut ll = LinkedList::from([2,3,4,5]);
    insert_just_before(&mut ll, &mut LinkedList::from([0]), &even);
    assert_eq!(ll, LinkedList::from([0,2,3,4,5]));
}