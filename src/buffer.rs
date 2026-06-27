//! Owned byte buffer types for storage I/O.
//!
//! [`Bytes`] is a zero-copy owned byte container that can be backed by a pooled,
//! memory-mapped, or plain `Vec` storage. All variants provide uniform read
//! access via `AsRef<[u8]>` and `Deref<Target = [u8]>`.
//!
//! Read-capable backends allocate through the internal crate-only `Bytes::allocate`
//! path, which reuses buffers from a process-wide pool.
//! Zero-length reads return an empty `Bytes::Vec` without touching the pool.

use std::sync::Arc;

#[cfg(any(feature = "sync", feature = "tokio", feature = "io-uring"))]
use zeropool::BufferPool;

// ============================================================================
// MmapRegion — Arc-backed memory-mapped file region
// ============================================================================

/// A memory-mapped file region backed by an `Arc<memmap2::Mmap>`.
///
/// Cheaply cloneable; `as_slice()` returns exactly `len` bytes starting at
/// `start` within the underlying mapping.
#[cfg(feature = "mmap")]
#[derive(Debug, Clone)]
pub struct MmapRegion {
    inner: Arc<memmap2::Mmap>,
    start: usize,
    len: usize,
}

#[cfg(feature = "mmap")]
impl MmapRegion {
    pub(crate) fn new(inner: Arc<memmap2::Mmap>, start: usize, len: usize) -> Self {
        debug_assert!(start.checked_add(len).is_some_and(|end| end <= inner.len()));
        Self { inner, start, len }
    }

    #[inline]
    #[must_use]
    pub fn subregion(&self, offset: usize, len: usize) -> Option<Self> {
        let relative_end = offset.checked_add(len)?;
        if relative_end > self.len {
            return None;
        }
        let start = self.start.checked_add(offset)?;
        Some(Self::new(Arc::clone(&self.inner), start, len))
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.inner[self.start..self.start + self.len]
    }
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(feature = "mmap")]
impl AsRef<[u8]> for MmapRegion {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

#[cfg(feature = "mmap")]
impl std::ops::Deref for MmapRegion {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

// ============================================================================
// Bytes
// ============================================================================

/// Owned byte buffer backed by one of several storage strategies.
///
/// All variants provide uniform read access via `AsRef<[u8]>` / `Deref`.
/// Only the `Pooled` and `Vec` variants support mutable access.
pub enum Bytes {
    /// A buffer returned from the internal process-wide pool.
    Pooled(zeropool::PooledBuffer),
    /// A memory-mapped file region (zero-copy, read-only).
    #[cfg(feature = "mmap")]
    Mmap(crate::mmap::MmapRegion),
    /// A plain heap-allocated buffer.
    Vec(Vec<u8>),
}

impl Bytes {
    /// Allocate a buffer for a read of `len` bytes.
    ///
    /// The returned buffer is not initialized; the caller must fill it through
    /// `as_mut_slice` before exposing it to consumers.
    #[cfg(any(feature = "sync", feature = "tokio", feature = "io-uring"))]
    #[inline]
    pub(crate) fn allocate(len: usize) -> Self {
        if len == 0 {
            return Self::Vec(Vec::new());
        }
        Self::Pooled(global_pool().get(len))
    }

    /// Wrap a plain `Vec<u8>`.
    #[inline]
    #[must_use]
    pub fn from_vec(v: Vec<u8>) -> Self {
        Self::Vec(v)
    }

    /// Number of bytes.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Pooled(b) => b.len(),
            #[cfg(feature = "mmap")]
            Self::Mmap(b) => b.len(),
            Self::Vec(b) => b.len(),
        }
    }

    /// Returns `true` if there are no bytes.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a mutable byte slice for the variants that own their memory.
    ///
    /// Returns `None` for [`Bytes::Mmap`], which is immutable.
    #[inline]
    #[must_use]
    pub fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        match self {
            Self::Pooled(b) => Some(b.as_mut_slice()),
            #[cfg(feature = "mmap")]
            Self::Mmap(_) => None,
            Self::Vec(b) => Some(b.as_mut_slice()),
        }
    }

    /// Convert to a `Vec<u8>`.
    ///
    /// Copies when backed by `Mmap` storage to avoid mismatched-allocator UB.
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        match self {
            Self::Pooled(b) => b.into_inner(),
            #[cfg(feature = "mmap")]
            Self::Mmap(b) => b.as_slice().to_vec(),
            Self::Vec(b) => b,
        }
    }

    /// Convert to an `Arc<[u8]>`.
    ///
    /// Copies when backed by `Mmap` storage.
    #[must_use]
    pub fn into_shared(self) -> Arc<[u8]> {
        match self {
            Self::Pooled(b) => b.into_inner().into(),
            #[cfg(feature = "mmap")]
            Self::Mmap(b) => Arc::from(b.as_slice()),
            Self::Vec(b) => b.into(),
        }
    }
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Pooled(b) => b.as_ref(),
            #[cfg(feature = "mmap")]
            Self::Mmap(b) => b.as_slice(),
            Self::Vec(b) => b.as_ref(),
        }
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bytes")
            .field("len", &self.len())
            .finish_non_exhaustive()
    }
}

impl From<Vec<u8>> for Bytes {
    #[inline]
    fn from(v: Vec<u8>) -> Self {
        Self::Vec(v)
    }
}

#[cfg(any(feature = "sync", feature = "tokio", feature = "io-uring"))]
fn global_pool() -> &'static BufferPool {
    use std::sync::OnceLock;
    static POOL: OnceLock<BufferPool> = OnceLock::new();
    POOL.get_or_init(|| {
        BufferPool::builder()
            .num_shards(8)
            .tls_cache_size(4)
            .max_buffers_per_shard(32)
            .min_buffer_size(1024 * 1024)
            .build()
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "mmap")]
    fn make_mmap_region() -> crate::mmap::MmapRegion {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"hello_mmap").unwrap();
        tmp.flush().unwrap();
        let file = std::fs::File::open(tmp.path()).unwrap();
        // SAFETY: the temp file remains alive for the duration of the mapping
        // setup, and memmap2 owns the resulting mapping independently.
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };
        crate::mmap::MmapRegion::new(Arc::new(mmap), 0, 10)
    }

    #[test]
    fn from_vec_roundtrips() {
        let data = vec![1u8, 2, 3];
        let ob = Bytes::from_vec(data.clone());
        assert_eq!(ob.as_ref(), &data[..]);
    }

    #[cfg(any(feature = "sync", feature = "tokio", feature = "io-uring"))]
    #[test]
    fn allocate_pooled_roundtrips() {
        let mut ob = Bytes::allocate(8);
        let buf = ob.as_mut_slice().unwrap();
        buf[..4].copy_from_slice(&[10, 20, 30, 40]);
        assert_eq!(&ob.as_ref()[..4], &[10, 20, 30, 40]);
    }

    #[test]
    fn vec_variant() {
        let data = vec![5u8, 6, 7];
        let ob = Bytes::from_vec(data.clone());
        assert_eq!(ob.as_ref(), &data[..]);
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_variant_accessible() {
        let ob = Bytes::Mmap(make_mmap_region());
        assert_eq!(ob.as_ref(), b"hello_mmap");
    }

    #[test]
    fn len_and_is_empty_vec() {
        let ob = Bytes::from_vec(vec![1, 2]);
        assert_eq!(ob.len(), 2);
        assert!(!ob.is_empty());
        let empty = Bytes::from_vec(vec![]);
        assert!(empty.is_empty());
    }

    #[test]
    fn zero_length_vec_has_mutable_empty_slice() {
        let mut ob = Bytes::from_vec(Vec::new());

        let slice = ob.as_mut_slice().unwrap();

        assert!(slice.is_empty());
        assert!(ob.is_empty());
    }

    #[test]
    fn deref_matches_as_ref() {
        let ob = Bytes::from_vec(vec![42u8; 8]);
        let via_deref: &[u8] = &ob;
        assert_eq!(via_deref, ob.as_ref());
    }

    #[test]
    fn debug_shows_len() {
        let ob = Bytes::from_vec(vec![0u8; 17]);
        assert!(format!("{ob:?}").contains("17"));
    }

    #[test]
    fn as_mut_slice_some_for_vec() {
        let mut ob = Bytes::from_vec(vec![0u8; 4]);
        ob.as_mut_slice().unwrap()[0] = 99;
        assert_eq!(ob.as_ref()[0], 99);
    }

    #[cfg(any(feature = "sync", feature = "tokio", feature = "io-uring"))]
    #[test]
    fn as_mut_slice_some_for_pooled() {
        let mut ob = Bytes::allocate(4);
        assert!(ob.as_mut_slice().is_some());
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn as_mut_slice_none_for_mmap() {
        let mut ob = Bytes::Mmap(make_mmap_region());
        assert!(ob.as_mut_slice().is_none());
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_subregion_returns_requested_window() {
        let region = make_mmap_region();

        let subregion = region.subregion(6, 4).unwrap();

        assert_eq!(subregion.as_slice(), b"mmap");
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_subregion_rejects_out_of_bounds_window() {
        let region = make_mmap_region();

        assert!(region.subregion(8, 3).is_none());
        assert!(region.subregion(usize::MAX, 1).is_none());
    }

    #[test]
    fn into_vec_preserves_bytes() {
        let data = vec![7u8, 8, 9];
        assert_eq!(Bytes::from_vec(data.clone()).into_vec(), data);
        #[cfg(feature = "mmap")]
        assert_eq!(Bytes::Mmap(make_mmap_region()).into_vec(), b"hello_mmap");
    }

    #[test]
    fn into_shared_preserves_bytes() {
        let data = vec![1u8, 2, 3];
        let shared = Bytes::from_vec(data.clone()).into_shared();
        assert_eq!(shared.as_ref(), &data[..]);
        #[cfg(feature = "mmap")]
        let shared2 = Bytes::Mmap(make_mmap_region()).into_shared();
        #[cfg(feature = "mmap")]
        assert_eq!(shared2.as_ref(), b"hello_mmap");
    }

    #[cfg(any(feature = "sync", feature = "tokio", feature = "io-uring"))]
    #[test]
    fn allocate_returns_pooled_storage() {
        let ob = Bytes::allocate(4);
        assert!(matches!(ob, Bytes::Pooled(_)));
        assert_eq!(ob.len(), 4);
    }

    #[cfg(any(feature = "sync", feature = "tokio", feature = "io-uring"))]
    #[test]
    fn zero_length_allocation_is_empty_vec() {
        let ob = Bytes::allocate(0);
        assert!(matches!(ob, Bytes::Vec(_)));
        assert!(ob.is_empty());
    }
}
