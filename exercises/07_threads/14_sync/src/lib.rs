// Not much to be exercised on `Sync`, just a thing to remember.
fn outro() -> &'static str {
    "I have a good understanding of Send and Sync!"
}

#[cfg(test)]
mod tests {
    use crate::outro;

    #[test]
    fn test_outro() {
        // MutexGuard does not meet the `Send` trait requirement as it is
        // **not safe** to transfer(move) the ownership to another thread.
        // Why **not safe**? The `lock` and `unlock` operations
        // should be invoked from the same thread.
        //
        // MutexGuard does meet the `Sync` trait requirement, because giving
        // a `&MutexGuard`(immutable shared reference) to another thread has
        // no impact on where the lock is released.
        //
        // RefCell does meet the `Send` trait requirement as it is **ok(safe)**
        // to move the ownership to another thread.
        //
        // RefCell does not meet `Sync` trait requirement as once two threads
        // get the same `&RefCelll`(immutable reference), one thred may change
        // the inner value due to interior mutability.
        assert_eq!(outro(), "I have a good understanding of Send and Sync!");
    }
}
