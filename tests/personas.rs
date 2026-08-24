//! External-style ownership patterns exercised through only the public API.

use std::{collections::VecDeque, rc::Rc};

use bytebudget::{ByteBudget, ByteCount, Retained};

fn count(bytes: usize) -> ByteCount {
    ByteCount::try_from(bytes).expect("test storage fits the accounting domain")
}

struct Payload {
    storage: Vec<u8>,
}

impl Retained for Payload {
    fn retained_bytes(&self) -> ByteCount {
        count(self.storage.capacity())
    }
}

struct Entry<T> {
    value: T,
    charged: ByteCount,
}

#[test]
fn bounded_mailbox_measures_once_and_releases_the_admission_charge() {
    let mut storage = Vec::with_capacity(32);
    storage.extend_from_slice(&[1, 2, 3]);
    let value = Payload { storage };
    let charged = value.retained_bytes();
    let mut budget = ByteBudget::new(charged);
    let mut mailbox = VecDeque::new();

    budget.try_reserve(charged).unwrap();
    mailbox.push_back(Entry { value, charged });
    assert!(budget.is_exhausted());

    let mut entry = mailbox.pop_front().expect("admitted entry exists");
    entry.value.storage = Vec::new();
    assert_eq!(entry.value.retained_bytes(), ByteCount::ZERO);
    budget.release(entry.charged).unwrap();
    assert_eq!(budget.used(), ByteCount::ZERO);
}

#[test]
fn transactional_admission_rolls_back_the_exact_first_charge() {
    let first = ByteCount::new(6);
    let second = ByteCount::new(5);
    let mut budget = ByteBudget::new(ByteCount::new(10));

    budget.try_reserve(first).unwrap();
    let second_error = budget.try_reserve(second).unwrap_err();
    assert_eq!(second_error.requested(), second);
    assert_eq!(second_error.available(), ByteCount::new(4));

    budget.release(first).unwrap();
    assert_eq!(budget.used(), ByteCount::ZERO);
    assert_eq!(budget.available(), ByteCount::new(10));
}

#[test]
fn partial_drain_releases_completed_segments_incrementally() {
    let segments = [ByteCount::new(5), ByteCount::new(3), ByteCount::new(4)];
    let total = segments
        .into_iter()
        .try_fold(ByteCount::ZERO, ByteCount::checked_add)
        .expect("test segment sum fits");
    let mut budget = ByteBudget::new(total);

    budget.try_reserve(total).unwrap();
    assert!(budget.is_exhausted());
    budget.release(segments[0]).unwrap();
    assert_eq!(budget.used(), ByteCount::new(7));
    budget.release(segments[1]).unwrap();
    assert_eq!(budget.used(), ByteCount::new(4));
    budget.release(segments[2]).unwrap();
    assert_eq!(budget.used(), ByteCount::ZERO);
}

struct SharedPayload {
    storage: Rc<Vec<u8>>,
}

impl Retained for SharedPayload {
    fn retained_bytes(&self) -> ByteCount {
        count(self.storage.capacity())
    }
}

#[test]
fn shared_backing_storage_is_conservatively_charged_per_reference() {
    let storage = Rc::new(Vec::with_capacity(24));
    let first = SharedPayload { storage: Rc::clone(&storage) };
    let second = SharedPayload { storage: Rc::clone(&storage) };
    let charge = first.retained_bytes();
    assert_eq!(second.retained_bytes(), charge);
    assert!(Rc::ptr_eq(&first.storage, &second.storage));

    let limit = charge.checked_add(charge).expect("test charge fits");
    let mut budget = ByteBudget::new(limit);
    budget.try_reserve(first.retained_bytes()).unwrap();
    budget.try_reserve(second.retained_bytes()).unwrap();

    assert_eq!(budget.used(), limit);
    assert_eq!(Rc::strong_count(&storage), 3);
}
