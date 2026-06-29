//! Small helper traits and wrapper types used across the crate.

use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Converts a serializable value into `serde_json::Value`.
///
/// The trait is blanket-implemented for all types, so any serializable value
/// can call `.json_value()` directly.
pub trait IntoJsonValue {
    /// Serializes `self` into a JSON value.
    ///
    /// # Errors
    ///
    /// Returns any serialization error produced by `serde_json`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::utils::helpers::IntoJsonValue;
    ///
    /// let value = vec!["a", "b"].json_value()?;
    /// assert!(value.is_array());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    fn json_value(self) -> serde_json::Result<Value>
    where
        Self: Sized + Serialize,
    {
        serde_json::to_value(self)
    }
}

impl<T> IntoJsonValue for T {}

#[derive(Clone, Debug)]
pub struct ArcRwLock<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> ArcRwLock<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.inner.write().await
    }
}

impl<T> ArcRwLock<T>
where
    T: Clone,
{
    pub async fn get_clone(&self) -> T {
        self.inner.read().await.clone()
    }
}
