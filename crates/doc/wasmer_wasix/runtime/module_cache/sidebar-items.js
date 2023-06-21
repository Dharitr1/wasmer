window.SIDEBAR_ITEMS = {"enum":[["CacheError","Possible errors that may occur during [`ModuleCache`] operations."]],"fn":[["in_memory","Get a [`ModuleCache`] which should be good enough for most in-memory use cases."]],"mod":[["fallback",""],["filesystem",""],["shared",""],["thread_local",""],["types",""]],"struct":[["FallbackCache","[`FallbackCache`] is a combinator for the [`ModuleCache`] trait that enables the chaining of two caching strategies together, typically via [`ModuleCache::with_fallback()`]."],["FileSystemCache","A cache that saves modules to a folder on the host filesystem using [`Module::serialize()`]."],["ModuleHash","The SHA-256 hash of a WebAssembly module."],["SharedCache","A [`ModuleCache`] based on a [DashMap]<[ModuleHash], [Module]>."],["ThreadLocalCache","A cache that will cache modules in a thread-local variable."]],"trait":[["ModuleCache","A cache for compiled WebAssembly modules."]]};