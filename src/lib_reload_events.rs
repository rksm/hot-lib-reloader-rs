use std::{
    borrow::BorrowMut,
    sync::{mpsc, Arc, Condvar, Mutex},
    time::Duration,
};

/// Signals when the library has changed.
/// Needs to be public as it is used in `hot_module`.
#[derive(Clone)]
#[non_exhaustive]
#[doc(hidden)]
pub enum ChangedEvent {
    LibAboutToReload(BlockReload),
    LibReloaded,
}

impl std::fmt::Debug for ChangedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibAboutToReload(_) => write!(f, "LibAboutToReload"),
            Self::LibReloaded => write!(f, "LibReloaded"),
        }
    }
}

/// See [`LibReloadObserver::wait_for_about_to_reload`].
///
/// [`BlockReload`] is implemented using a simple counting scheme to track how
/// many tokens are floating around. If the number reaches 0 the update can
/// continue.
#[derive(Debug)]
pub struct BlockReload {
    pub(crate) pending: Arc<(Mutex<usize>, Condvar)>,
}

impl Clone for BlockReload {
    fn clone(&self) -> Self {
        **(self.pending.0.lock().unwrap().borrow_mut()) += 1;
        Self {
            pending: self.pending.clone(),
        }
    }
}

impl Drop for BlockReload {
    fn drop(&mut self) {
        let (counter, cond) = &*self.pending;
        *counter.lock().unwrap() -= 1;
        cond.notify_one();
    }
}

/// A [`LibReloadObserver`] allows to wait for library changes. See
/// - [`LibReloadObserver::wait_for_about_to_reload`] and
/// - [`LibReloadObserver::wait_for_reload`]
/// for details.
///
/// You can use those methods individually or in combination. In particular, if you want to serialize state before a library change happens and then deserialize / migrate it when the library update is done, using both methods in combination is quite useful. Something along the lines of:
///
/// ```ignore
/// #[hot_module(dylib = "lib")]
/// mod hot_lib {
///     #[lib_change_subscription]
///     pub fn subscribe() -> hot_lib_reloader::LibReloadObserver { }
/// }
///
/// fn test() {
///     let lib_observer = hot_lib::subscribe();
///
///     /* ... */
///
///     // wait for reload to begin (at this point the  old version is still loaded)
///     let update_blocker = lib_observer.wait_for_about_to_reload();
///
///     /* do update preparations here, e.g. serialize state */
///
///     // drop the blocker to allow update
///     drop(update_blocker);
///
///     // wait for reload to be completed
///     lib_observer.wait_for_reload();
///
///     /* new lib version is loaded now so you can e.g. restore state */
/// }
/// ```
pub struct LibReloadObserver {
    // needs to be public b/c it is used inside the [`hot_module`] macro.
    #[doc(hidden)]
    pub rx: mpsc::Receiver<ChangedEvent>,
}

impl LibReloadObserver {
    /// A call to this method will do a blocking wait until the watched library is
    /// about to change. It returns a [`BlockReload`] token. While this token is in
    /// scope you will prevent the pending update to proceed. This is useful for
    /// doing preparations for the update and while the old library version is still
    /// loaded. You can for example serialize state.
    pub fn wait_for_about_to_reload(&self) -> BlockReload {
        loop {
            match self.rx.recv() {
                Ok(ChangedEvent::LibAboutToReload(block)) => return block,
                Err(err) => {
                    panic!("LibReloadObserver failed to wait for event from reloader: {err}")
                }
                _ => continue,
            }
        }
    }

    /// Like [`Self::wait_for_about_to_reload`] but for a limited time. In case of a timeout return `None`.
    pub fn wait_for_about_to_reload_timeout(&self, timeout: Duration) -> Option<BlockReload> {
        loop {
            match self.rx.recv_timeout(timeout) {
                Ok(ChangedEvent::LibAboutToReload(block)) => return Some(block),
                Err(_) => return None,
                _ => continue,
            }
        }
    }

    /// Will do blocking wait until a new library version is loaded.
    pub fn wait_for_reload(&self) {
        loop {
            match self.rx.recv() {
                Ok(ChangedEvent::LibReloaded) => return,
                Err(err) => {
                    panic!("LibReloadObserver failed to wait for event from reloader: {err}")
                }
                _ => continue,
            }
        }
    }

    /// Like [`Self::wait_for_reload`] but for a limited time. In case of a timeout return `false`.
    pub fn wait_for_reload_timeout(&self, timeout: Duration) -> bool {
        loop {
            match self.rx.recv_timeout(timeout) {
                Ok(ChangedEvent::LibReloaded) => return true,
                Err(_) => return false,
                _ => continue,
            }
        }
    }
}

/// Needs to be public as it is used in the `hot_module` macro.
#[derive(Default)]
#[doc(hidden)]
pub struct LibReloadNotifier {
    subscribers: Arc<Mutex<Vec<mpsc::Sender<ChangedEvent>>>>,
}

impl LibReloadNotifier {
    /// Needs to be public as it is used in the `hot_module` macro.
    ///
    /// The count used here represents [`BlockReload`] tokens that are still
    /// floating around. When a token is dropped the count is decremented and
    /// the condvar signaled.
    #[doc(hidden)]
    pub fn send_about_to_reload_event_and_wait_for_blocks(&self) {
        let pending = Arc::new((Mutex::new(1), std::sync::Condvar::new()));
        let block = BlockReload {
            pending: pending.clone(),
        };
        self.notify(ChangedEvent::LibAboutToReload(block));
        let (counter, cond) = &*pending;
        log::trace!(
            "about-to-change library event, waiting for {}",
            counter.lock().unwrap()
        );
        let _guard = cond
            .wait_while(counter.lock().unwrap(), |pending| {
                log::trace!(
                    "about-to-change library event, now waiting for {}",
                    *pending
                );
                *pending > 0
            })
            .unwrap();
    }

    #[doc(hidden)]
    pub fn send_reloaded_event(&self) {
        self.notify(ChangedEvent::LibReloaded);
    }

    fn notify(&self, evt: ChangedEvent) {
        if let Ok(mut subscribers) = self.subscribers.try_lock() {
            let n = subscribers.len();
            log::trace!("sending {evt:?} to {n} subscribers");
            // keep only those subscribers that are still around and kicking.
            subscribers.retain(|tx| tx.send(evt.clone()).is_ok());
            let removed = n - subscribers.len();
            if removed > 0 {
                log::debug!(
                    "removing {removed} subscriber{}",
                    if removed == 1 { "" } else { "s" }
                );
            }
        }
    }

    /// Needs to be public as it is used in the `hot_module` macro.
    ///
    /// Create a [ChangedEvent] receiver that gets signalled when the library
    /// changes.
    #[doc(hidden)]
    pub fn subscribe(&mut self) -> LibReloadObserver {
        log::trace!("subscribe to lib change");
        let (tx, rx) = mpsc::channel();
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push(tx);
        LibReloadObserver { rx }
    }
}
