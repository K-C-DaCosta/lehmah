use super::*;

#[test]
fn sanity_checks() {
    let mut rb = RingBuffer::with_backing([0usize; 9]);
    assert_eq!(
        rb.dequeue(),
        Err(RingError::BufferEmpty),
        "Unexpected dequeue result"
    );
    assert_eq!(rb.enqueue(1), Ok(()));
    assert_eq!(rb.enqueue(2), Ok(()));
    assert_eq!(rb.len(), 2, "Unexpected length");
    assert_eq!(rb.enqueue(3), Ok(()));
    assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(rb.enqueue(4), Ok(()));
    assert_eq!(rb.enqueue(5), Ok(()));
    assert_eq!(rb.enqueue(6), Ok(()));
    assert_eq!(rb.enqueue(7), Ok(()));
    assert_eq!(rb.enqueue(8), Ok(()));
    assert_eq!(
        rb.iter().copied().collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(rb.enqueue(9), Ok(()));
    assert_eq!(rb.enqueue(10), Err(RingError::BufferFull));

    assert_eq!(rb.dequeue().copied(), Ok(1));
    assert_eq!(
        rb.iter().copied().collect::<Vec<_>>(),
        vec![2, 3, 4, 5, 6, 7, 8, 9]
    );
}
