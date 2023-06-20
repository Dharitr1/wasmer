use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    path::PathBuf,
};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use semver::Version;

use crate::runtime::resolver::{
    outputs::{Edge, Node},
    DependencyGraph, ItemLocation, PackageId, PackageInfo, PackageSpecifier, PackageSummary,
    QueryError, Resolution, ResolvedPackage, Source,
};

use super::ResolvedFileSystemMapping;

/// Given the [`PackageInfo`] for a root package, resolve its dependency graph
/// and figure out how it could be executed.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn resolve(
    root_id: &PackageId,
    root: &PackageInfo,
    source: &dyn Source,
) -> Result<Resolution, ResolveError> {
    let graph = resolve_dependency_graph(root_id, root, source).await?;
    let package = resolve_package(&graph)?;

    Ok(Resolution { graph, package })
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("{}", registry_error_message(.package))]
    Registry {
        package: PackageSpecifier,
        #[source]
        error: QueryError,
    },
    #[error("Dependency cycle detected: {}", print_cycle(_0))]
    Cycle(Vec<PackageId>),
    #[error(
        "Multiple versions of {package_name} were found {}",
        versions.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "),
    )]
    DuplicateVersions {
        package_name: String,
        versions: Vec<Version>,
    },
}

fn registry_error_message(specifier: &PackageSpecifier) -> String {
    match specifier {
        PackageSpecifier::Registry { full_name, version } if version.comparators.is_empty() => {
            format!("Unable to find \"{full_name}\" in the registry")
        }
        PackageSpecifier::Registry { full_name, version } => {
            format!("Unable to find \"{full_name}@{version}\" in the registry")
        }
        PackageSpecifier::Url(url) => format!("Unable to resolve \"{url}\""),
        PackageSpecifier::Path(path) => {
            format!("Unable to load \"{}\" from disk", path.display())
        }
    }
}

impl ResolveError {
    pub fn as_cycle(&self) -> Option<&[PackageId]> {
        match self {
            ResolveError::Cycle(cycle) => Some(cycle),
            _ => None,
        }
    }
}

fn print_cycle(packages: &[PackageId]) -> String {
    packages
        .iter()
        .map(|pkg_id| {
            let PackageId {
                package_name,
                version,
                ..
            } = pkg_id;
            format!("{package_name}@{version}")
        })
        .collect::<Vec<_>>()
        .join(" → ")
}

async fn resolve_dependency_graph(
    root_id: &PackageId,
    root: &PackageInfo,
    source: &dyn Source,
) -> Result<DependencyGraph, ResolveError> {
    let DiscoveredPackages {
        root,
        graph,
        indices,
        packages,
    } = discover_dependencies(root_id, root, source).await?;

    check_for_duplicate_versions(indices.iter().copied().map(|ix| &graph[ix].id))?;
    log_dependencies(&graph, root);

    let graph = DependencyGraph::new(root, graph, packages);

    Ok(graph)
}

async fn discover_dependencies(
    root_id: &PackageId,
    root: &PackageInfo,
    source: &dyn Source,
) -> Result<DiscoveredPackages, ResolveError> {
    let mut nodes: BTreeMap<PackageId, NodeIndex> = BTreeMap::new();
    let mut graph: DiGraph<Node, Edge> = DiGraph::new();

    let root_index = graph.add_node(Node {
        id: root_id.clone(),
        pkg: root.clone(),
        dist: None,
    });
    nodes.insert(root_id.clone(), root_index);

    let mut to_visit = VecDeque::new();
    to_visit.push_back(root_index);

    while let Some(index) = to_visit.pop_front() {
        let mut to_add = Vec::new();

        for dep in &graph[index].pkg.dependencies {
            // Get the latest version that satisfies our requirement. If we were
            // doing this more rigorously, we would be narrowing the version
            // down using existing requirements and trying to reuse the same
            // dependency when possible.
            let dep_summary =
                source
                    .latest(&dep.pkg)
                    .await
                    .map_err(|error| ResolveError::Registry {
                        package: dep.pkg.clone(),
                        error,
                    })?;
            let dep_id = dep_summary.package_id();

            let PackageSummary { pkg, dist } = dep_summary;

            let alias = dep.alias().to_string();
            let node = Node {
                id: dep_id,
                pkg,
                dist: Some(dist),
            };
            // Note: We can't add the node to the graph directly because we're
            // still iterating over it.
            to_add.push((alias, node));
        }

        for (alias, node) in to_add {
            let dep_id = node.id.clone();

            let dep_index = match nodes.get(&dep_id) {
                Some(&ix) => ix,
                None => {
                    // Create a new node and schedule its dependencies to be
                    // retrieved
                    let ix = graph.add_node(node);
                    nodes.insert(dep_id, ix);
                    to_visit.push_back(ix);
                    ix
                }
            };

            graph.add_edge(index, dep_index, Edge { alias });
        }
    }

    let sorted_indices = petgraph::algo::toposort(&graph, None).map_err(|_| cycle_error(&graph))?;

    Ok(DiscoveredPackages {
        root: root_index,
        graph,
        indices: sorted_indices,
        packages: nodes,
    })
}

fn cycle_error(graph: &petgraph::Graph<Node, Edge>) -> ResolveError {
    // We know the graph has at least one cycle, so use SCC to find it.
    let mut cycle = petgraph::algo::kosaraju_scc(graph)
        .into_iter()
        .find(|cycle| cycle.len() > 1)
        .expect("We know there is at least one cycle");

    // we want the loop's starting node to be deterministic (for tests), and
    // nodes with lower indices are normally closer to the root of the
    // dependency tree.
    let lowest_index_node = cycle.iter().copied().min().expect("Cycle is non-empty");

    // We want the cycle vector to start with that node, so let's do a bit of
    // shuffling
    let offset = cycle
        .iter()
        .position(|&node| node == lowest_index_node)
        .unwrap();
    cycle.rotate_left(offset);

    // Don't forget to make the cycle start and end with the same node
    cycle.push(lowest_index_node);

    let package_ids = cycle.into_iter().map(|ix| graph[ix].pkg.id()).collect();
    ResolveError::Cycle(package_ids)
}

#[derive(Debug)]
struct DiscoveredPackages {
    root: NodeIndex,
    graph: DiGraph<Node, Edge>,
    /// All node indices, in topologically sorted order.
    indices: Vec<NodeIndex>,
    packages: BTreeMap<PackageId, NodeIndex>,
}

#[tracing::instrument(level = "debug", name = "dependencies", skip_all)]
fn log_dependencies(graph: &DiGraph<Node, Edge>, root: NodeIndex) {
    tracing::debug!(
        root = root.index(),
        dependency_count = graph.node_count(),
        "Resolved dependencies",
    );

    if tracing::enabled!(tracing::Level::TRACE) {
        petgraph::visit::depth_first_search(graph, [root], |event| {
            if let petgraph::visit::DfsEvent::Discover(n, _) = event {
                let package = &graph[n].id;
                let dependencies: BTreeMap<_, _> = graph
                    .edges(n)
                    .map(|edge_ref| (&edge_ref.weight().alias, &graph[edge_ref.target()].id))
                    .collect();

                tracing::trace!(%package, ?dependencies);
            }
        });
    }
}

/// As a workaround for the lack of "proper" dependency merging, we'll make sure
/// only one copy of each package is in the dependency tree. If the same package
/// is included in the tree multiple times, they all need to use the exact same
/// version otherwise it's an error.
fn check_for_duplicate_versions<'a, I>(package_ids: I) -> Result<(), ResolveError>
where
    I: Iterator<Item = &'a PackageId>,
{
    let mut package_versions: BTreeMap<&str, HashSet<&Version>> = BTreeMap::new();

    for PackageId {
        package_name,
        version,
    } in package_ids
    {
        package_versions
            .entry(package_name)
            .or_default()
            .insert(version);
    }

    for (package_name, versions) in package_versions {
        if versions.len() > 1 {
            let mut versions: Vec<_> = versions.into_iter().cloned().collect();
            versions.sort();
            return Err(ResolveError::DuplicateVersions {
                package_name: package_name.to_string(),
                versions,
            });
        }
    }

    Ok(())
}

/// Given some [`DiscoveredPackages`], figure out how the resulting "package"
/// would look when loaded at runtime.
fn resolve_package(dependency_graph: &DependencyGraph) -> Result<ResolvedPackage, ResolveError> {
    // FIXME: This code is all super naive and will break the moment there
    // are any conflicts or duplicate names.
    tracing::trace!("Resolving the package");

    let mut commands = BTreeMap::new();
    let mut filesystem = Vec::new();

    let mut entrypoint = dependency_graph.root_info().entrypoint.clone();

    for index in petgraph::algo::toposort(dependency_graph.graph(), None).expect("acyclic") {
        let node = &dependency_graph[index];
        let id = &node.id;
        let pkg = &node.pkg;

        // update the entrypoint, if necessary
        if entrypoint.is_none() {
            if let Some(entry) = &pkg.entrypoint {
                tracing::trace!(
                    entrypoint = entry.as_str(),
                    parent.name=id.package_name.as_str(),
                    parent.version=%id.version,
                    "Inheriting the entrypoint",
                );

                entrypoint = Some(entry.clone());
            }
        }

        for cmd in &pkg.commands {
            // Note: We are traversing in topological order with the root at the
            // start, so if we ever see any duplicates we should prefer the
            // earlier copy and skip the later one.

            match commands.entry(cmd.name.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    let resolved = ItemLocation {
                        name: cmd.name.clone(),
                        package: id.clone(),
                    };
                    entry.insert(resolved);
                    tracing::trace!(
                        command.name=cmd.name.as_str(),
                        pkg.name=id.package_name.as_str(),
                        pkg.version=%id.version,
                        "Discovered command",
                    );
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    tracing::trace!(
                        command.name=cmd.name.as_str(),
                        pkg.name=id.package_name.as_str(),
                        pkg.version=%id.version,
                        "Ignoring duplicate command",
                    );
                }
            }
        }

        for mapping in &pkg.filesystem {
            let dep = match &mapping.dependency_name {
                Some(name) => {
                    let dep_index = dependency_graph
                        .graph()
                        .edges(index)
                        .find(|edge| edge.weight().alias == *name)
                        .unwrap()
                        .target();
                    &dependency_graph[dep_index].id
                }
                None => id,
            };
            filesystem.push(ResolvedFileSystemMapping {
                mount_path: PathBuf::from(&mapping.mount_path),
                original_path: mapping.original_path.clone(),
                volume_name: mapping.volume_name.clone(),
                package: dep.clone(),
            })
        }
    }

    // Note: when resolving filesystem mappings, the first mapping will come
    // from the root package and its dependencies will be following. However, we
    // actually want things closer to the root package in the dependency tree to
    // come later so they override their dependencies.
    filesystem.reverse();

    Ok(ResolvedPackage {
        root_package: dependency_graph.root_info().id(),
        commands,
        entrypoint,
        filesystem,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::runtime::resolver::{
        inputs::{DistributionInfo, FileSystemMapping, PackageInfo},
        Dependency, InMemorySource, MultiSource, PackageSpecifier,
    };

    use super::*;

    struct RegistryBuilder(InMemorySource);

    impl RegistryBuilder {
        fn new() -> Self {
            RegistryBuilder(InMemorySource::new())
        }

        fn register(&mut self, name: &str, version: &str) -> AddPackageVersion<'_> {
            let pkg = PackageInfo {
                name: name.to_string(),
                version: version.parse().unwrap(),
                dependencies: Vec::new(),
                commands: Vec::new(),
                entrypoint: None,
                filesystem: Vec::new(),
            };
            let dist = DistributionInfo {
                webc: format!("http://localhost/{name}@{version}")
                    .parse()
                    .unwrap(),
                webc_sha256: [0; 32].into(),
            };
            let summary = PackageSummary { pkg, dist };

            AddPackageVersion {
                builder: &mut self.0,
                summary,
            }
        }

        fn finish(&self) -> MultiSource {
            let mut registry = MultiSource::new();
            registry.add_source(self.0.clone());
            registry
        }

        fn get(&self, package: &str, version: &str) -> &PackageSummary {
            let version = version.parse().unwrap();
            self.0.get(package, &version).unwrap()
        }

        fn start_dependency_graph(&self) -> DependencyGraphBuilder<'_> {
            DependencyGraphBuilder {
                dependencies: BTreeMap::new(),
                source: &self.0,
            }
        }
    }

    #[derive(Debug)]
    struct AddPackageVersion<'builder> {
        builder: &'builder mut InMemorySource,
        summary: PackageSummary,
    }

    impl<'builder> AddPackageVersion<'builder> {
        fn with_dependency(&mut self, name: &str, version_constraint: &str) -> &mut Self {
            self.with_aliased_dependency(name, name, version_constraint)
        }

        fn with_aliased_dependency(
            &mut self,
            alias: &str,
            name: &str,
            version_constraint: &str,
        ) -> &mut Self {
            let pkg = PackageSpecifier::Registry {
                full_name: name.to_string(),
                version: version_constraint.parse().unwrap(),
            };

            self.summary.pkg.dependencies.push(Dependency {
                alias: alias.to_string(),
                pkg,
            });

            self
        }

        fn with_command(&mut self, name: &str) -> &mut Self {
            self.summary
                .pkg
                .commands
                .push(crate::runtime::resolver::Command {
                    name: name.to_string(),
                });
            self
        }

        fn with_entrypoint(&mut self, name: &str) -> &mut Self {
            self.summary.pkg.entrypoint = Some(name.to_string());
            self
        }

        fn with_fs_mapping(
            &mut self,
            volume_name: &str,
            original_path: &str,
            mount_path: &str,
        ) -> &mut Self {
            self.summary.pkg.filesystem.push(FileSystemMapping {
                volume_name: volume_name.to_string(),
                mount_path: mount_path.to_string(),
                original_path: original_path.to_string(),
                dependency_name: None,
            });
            self
        }

        fn with_fs_mapping_from_dependency(
            &mut self,
            volume_name: &str,
            mount_path: &str,
            original_path: &str,
            dependency: &str,
        ) -> &mut Self {
            self.summary.pkg.filesystem.push(FileSystemMapping {
                volume_name: volume_name.to_string(),
                mount_path: mount_path.to_string(),
                original_path: original_path.to_string(),
                dependency_name: Some(dependency.to_string()),
            });
            self
        }
    }

    impl<'builder> Drop for AddPackageVersion<'builder> {
        fn drop(&mut self) {
            let summary = self.summary.clone();
            self.builder.add(summary);
        }
    }

    #[derive(Debug)]
    struct DependencyGraphBuilder<'source> {
        dependencies: BTreeMap<PackageId, BTreeMap<String, PackageId>>,
        source: &'source InMemorySource,
    }

    impl<'source> DependencyGraphBuilder<'source> {
        fn insert(
            &mut self,
            package: &str,
            version: &str,
        ) -> DependencyGraphEntryBuilder<'source, '_> {
            let version = version.parse().unwrap();
            let pkg_id = self.source.get(package, &version).unwrap().package_id();
            DependencyGraphEntryBuilder {
                builder: self,
                pkg_id,
                dependencies: BTreeMap::new(),
            }
        }

        fn finish(self) -> BTreeMap<PackageId, BTreeMap<String, PackageId>> {
            self.dependencies
        }

        /// Using the dependency mapping that we've been building up, construct
        /// a dependency graph using the specified root package.
        fn graph(self, root_name: &str, version: &str) -> DependencyGraph {
            let version = version.parse().unwrap();
            let root_id = self.source.get(root_name, &version).unwrap().package_id();

            let mut graph = DiGraph::new();
            let mut nodes = BTreeMap::new();

            for id in self.dependencies.keys() {
                let PackageSummary { pkg, dist } =
                    self.source.get(&id.package_name, &id.version).unwrap();
                let index = graph.add_node(Node {
                    id: pkg.id(),
                    pkg: pkg.clone(),
                    dist: Some(dist.clone()),
                });
                nodes.insert(id.clone(), index);
            }

            for (id, deps) in &self.dependencies {
                let index = nodes[id];
                for (dep_name, dep_id) in deps {
                    let dep_index = nodes[dep_id];
                    graph.add_edge(
                        index,
                        dep_index,
                        Edge {
                            alias: dep_name.clone(),
                        },
                    );
                }
            }

            let root_index = nodes[&root_id];

            DependencyGraph::new(root_index, graph, nodes)
        }
    }

    #[derive(Debug)]
    struct DependencyGraphEntryBuilder<'source, 'builder> {
        builder: &'builder mut DependencyGraphBuilder<'source>,
        pkg_id: PackageId,
        dependencies: BTreeMap<String, PackageId>,
    }

    impl<'source, 'builder> DependencyGraphEntryBuilder<'source, 'builder> {
        fn with_dependency(&mut self, name: &str, version: &str) -> &mut Self {
            self.with_aliased_dependency(name, name, version)
        }

        fn with_aliased_dependency(&mut self, alias: &str, name: &str, version: &str) -> &mut Self {
            let version = version.parse().unwrap();
            let dep_id = self
                .builder
                .source
                .get(name, &version)
                .unwrap()
                .package_id();
            self.dependencies.insert(alias.to_string(), dep_id);
            self
        }
    }

    impl<'source, 'builder> Drop for DependencyGraphEntryBuilder<'source, 'builder> {
        fn drop(&mut self) {
            self.builder
                .dependencies
                .insert(self.pkg_id.clone(), self.dependencies.clone());
        }
    }

    macro_rules! map {
        (
            $(
                $key:expr => $value:expr
            ),*
            $(,)?
        ) => {
            vec![
                $( ($key.into(), $value.into()) ),*
            ]
            .into_iter()
            .collect()
        }
    }

    fn deps(resolution: &Resolution) -> BTreeMap<PackageId, BTreeMap<String, PackageId>> {
        resolution
            .graph
            .iter_dependencies()
            .map(|(id, deps)| {
                let deps = deps
                    .into_iter()
                    .map(|(name, dep_id)| (name.to_string(), dep_id.clone()))
                    .collect();
                (id.clone(), deps)
            })
            .collect()
    }

    #[tokio::test]
    async fn no_deps_and_no_commands() {
        let mut builder = RegistryBuilder::new();
        builder.register("root", "1.0.0");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        let mut dependency_graph = builder.start_dependency_graph();
        dependency_graph.insert("root", "1.0.0");
        assert_eq!(deps(&resolution), dependency_graph.finish());
        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: BTreeMap::new(),
                entrypoint: None,
                filesystem: Vec::new(),
            }
        );
    }

    #[tokio::test]
    async fn no_deps_one_command() {
        let mut builder = RegistryBuilder::new();
        builder.register("root", "1.0.0").with_command("asdf");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        let mut dependency_graph = builder.start_dependency_graph();
        dependency_graph.insert("root", "1.0.0");
        assert_eq!(deps(&resolution), dependency_graph.finish());
        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: map! {
                    "asdf" => ItemLocation {
                        name: "asdf".to_string(),
                        package: root.package_id(),
                    },
                },
                entrypoint: None,
                filesystem: Vec::new(),
            }
        );
    }

    #[tokio::test]
    async fn single_dependency() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("dep", "=1.0.0");
        builder.register("dep", "1.0.0");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        let mut dependency_graph = builder.start_dependency_graph();
        dependency_graph
            .insert("root", "1.0.0")
            .with_dependency("dep", "1.0.0");
        dependency_graph.insert("dep", "1.0.0");
        assert_eq!(deps(&resolution), dependency_graph.finish());
        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: BTreeMap::new(),
                entrypoint: None,
                filesystem: Vec::new(),
            }
        );
    }

    #[tokio::test]
    async fn linear_dependency_chain() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("first", "1.0.0")
            .with_dependency("second", "=1.0.0");
        builder
            .register("second", "1.0.0")
            .with_dependency("third", "=1.0.0");
        builder.register("third", "1.0.0");
        let registry = builder.finish();
        let root = builder.get("first", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        let mut dependency_graph = builder.start_dependency_graph();
        dependency_graph
            .insert("first", "1.0.0")
            .with_dependency("second", "1.0.0");
        dependency_graph
            .insert("second", "1.0.0")
            .with_dependency("third", "1.0.0");
        dependency_graph.insert("third", "1.0.0");
        assert_eq!(deps(&resolution), dependency_graph.finish());
        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: BTreeMap::new(),
                entrypoint: None,
                filesystem: Vec::new(),
            }
        );
    }

    #[tokio::test]
    async fn pick_the_latest_dependency_when_multiple_are_possible() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("dep", "^1.0.0");
        builder.register("dep", "1.0.0");
        builder.register("dep", "1.0.1");
        builder.register("dep", "1.0.2");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        let mut dependency_graph = builder.start_dependency_graph();
        dependency_graph
            .insert("root", "1.0.0")
            .with_dependency("dep", "1.0.2");
        dependency_graph.insert("dep", "1.0.2");
        assert_eq!(deps(&resolution), dependency_graph.finish());
        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: BTreeMap::new(),
                entrypoint: None,
                filesystem: Vec::new(),
            }
        );
    }

    #[tokio::test]
    async fn version_merging_isnt_implemented_yet() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("first", "=1.0.0")
            .with_dependency("second", "=1.0.0");
        builder
            .register("first", "1.0.0")
            .with_dependency("common", "^1.0.0");
        builder
            .register("second", "1.0.0")
            .with_dependency("common", ">1.1,<1.3");
        builder.register("common", "1.0.0");
        builder.register("common", "1.1.0");
        builder.register("common", "1.2.0");
        builder.register("common", "1.5.0");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let result = resolve(&root.package_id(), &root.pkg, &registry).await;

        match result {
            Err(ResolveError::DuplicateVersions {
                package_name,
                versions,
            }) => {
                assert_eq!(package_name, "common");
                assert_eq!(
                    versions,
                    [
                        Version::parse("1.2.0").unwrap(),
                        Version::parse("1.5.0").unwrap(),
                    ]
                );
            }
            _ => unreachable!("Expected a duplicate versions error, found {:?}", result),
        }
    }

    #[tokio::test]
    #[ignore = "Version merging isn't implemented"]
    async fn merge_compatible_versions() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("first", "=1.0.0")
            .with_dependency("second", "=1.0.0");
        builder
            .register("first", "1.0.0")
            .with_dependency("common", "^1.0.0");
        builder
            .register("second", "1.0.0")
            .with_dependency("common", ">1.1,<1.3");
        builder.register("common", "1.0.0");
        builder.register("common", "1.1.0");
        builder.register("common", "1.2.0");
        builder.register("common", "1.5.0");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        let mut dependency_graph = builder.start_dependency_graph();
        dependency_graph
            .insert("root", "1.0.0")
            .with_dependency("first", "1.0.0")
            .with_dependency("second", "1.0.0");
        dependency_graph
            .insert("first", "1.0.0")
            .with_dependency("common", "1.2.0");
        dependency_graph
            .insert("second", "1.0.0")
            .with_dependency("common", "1.2.0");
        dependency_graph.insert("common", "1.2.0");
        assert_eq!(deps(&resolution), dependency_graph.finish());
        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: BTreeMap::new(),
                entrypoint: None,
                filesystem: Vec::new(),
            }
        );
    }

    #[tokio::test]
    async fn commands_from_dependencies_end_up_in_the_package() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("first", "=1.0.0")
            .with_dependency("second", "=1.0.0");
        builder
            .register("first", "1.0.0")
            .with_command("first-command");
        builder
            .register("second", "1.0.0")
            .with_command("second-command");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        let mut dependency_graph = builder.start_dependency_graph();
        dependency_graph
            .insert("root", "1.0.0")
            .with_dependency("first", "1.0.0")
            .with_dependency("second", "1.0.0");
        dependency_graph.insert("first", "1.0.0");
        dependency_graph.insert("second", "1.0.0");
        assert_eq!(deps(&resolution), dependency_graph.finish());
        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: map! {
                    "first-command" => ItemLocation {
                        name: "first-command".to_string(),
                        package: builder.get("first", "1.0.0").package_id(),
                     },
                    "second-command" => ItemLocation {
                        name: "second-command".to_string(),
                        package: builder.get("second", "1.0.0").package_id(),
                     },
                },
                entrypoint: None,
                filesystem: Vec::new(),
            }
        );
    }

    #[tokio::test]
    #[ignore = "TODO: Re-order the way commands are resolved"]
    async fn commands_in_root_shadow_their_dependencies() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("dep", "=1.0.0")
            .with_command("command");
        builder.register("dep", "1.0.0").with_command("command");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        let mut dependency_graph = builder.start_dependency_graph();
        dependency_graph
            .insert("root", "1.0.0")
            .with_dependency("dep", "1.0.0");
        dependency_graph.insert("dep", "1.0.0");
        assert_eq!(deps(&resolution), dependency_graph.finish());
        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: map! {
                    "command" => ItemLocation {
                        name: "command".to_string(),
                        package: builder.get("root", "1.0.0").package_id(),
                     },
                },
                entrypoint: None,
                filesystem: Vec::new(),
            }
        );
    }

    #[tokio::test]
    async fn cyclic_dependencies() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("dep", "=1.0.0");
        builder
            .register("dep", "1.0.0")
            .with_dependency("root", "=1.0.0");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let err = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap_err();

        let cycle = err.as_cycle().unwrap().to_vec();
        assert_eq!(
            cycle,
            [
                builder.get("root", "1.0.0").package_id(),
                builder.get("dep", "1.0.0").package_id(),
                builder.get("root", "1.0.0").package_id(),
            ]
        );
    }

    #[tokio::test]
    async fn entrypoint_is_inherited() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("dep", "=1.0.0");
        builder
            .register("dep", "1.0.0")
            .with_command("entry")
            .with_entrypoint("entry");
        let registry = builder.finish();
        let root = builder.get("root", "1.0.0");

        let resolution = resolve(&root.package_id(), &root.pkg, &registry)
            .await
            .unwrap();

        assert_eq!(
            resolution.package,
            ResolvedPackage {
                root_package: root.package_id(),
                commands: map! {
                    "entry" => ItemLocation {
                        name: "entry".to_string(),
                        package: builder.get("dep", "1.0.0").package_id(),
                     },
                },
                entrypoint: Some("entry".to_string()),
                filesystem: Vec::new(),
            }
        );
    }

    #[test]
    fn cyclic_error_message() {
        let cycle = [
            PackageId {
                package_name: "root".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            PackageId {
                package_name: "dep".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            PackageId {
                package_name: "root".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
        ];

        let message = print_cycle(&cycle);

        assert_eq!(message, "root@1.0.0 → dep@1.0.0 → root@1.0.0");
    }

    #[test]
    fn filesystem_with_one_package_and_no_fs_tables() {
        let mut builder = RegistryBuilder::new();
        builder.register("root", "1.0.0");
        let mut dep_builder = builder.start_dependency_graph();
        dep_builder.insert("root", "1.0.0");
        let graph = dep_builder.graph("root", "1.0.0");

        let pkg = resolve_package(&graph).unwrap();

        assert!(pkg.filesystem.is_empty());
    }

    #[test]
    fn filesystem_with_one_package_and_one_fs_tables() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_fs_mapping("atom", "/publisher/lib", "/lib");
        let mut dep_builder = builder.start_dependency_graph();
        dep_builder.insert("root", "1.0.0");
        let graph = dep_builder.graph("root", "1.0.0");

        let pkg = resolve_package(&graph).unwrap();

        assert_eq!(
            pkg.filesystem,
            vec![ResolvedFileSystemMapping {
                mount_path: PathBuf::from("/lib"),
                original_path: "/publisher/lib".to_string(),
                volume_name: "atom".to_string(),
                package: builder.get("root", "1.0.0").package_id(),
            }]
        );
    }

    #[test]
    fn merge_fs_mappings_from_multiple_packages() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("first", "=1.0.0")
            .with_dependency("second", "=1.0.0")
            .with_fs_mapping("atom", "/root", "/root");
        builder.register("first", "1.0.0").with_fs_mapping(
            "atom",
            "/usr/local/lib/first",
            "/usr/local/lib/first",
        );
        builder.register("second", "1.0.0").with_fs_mapping(
            "atom",
            "/usr/local/lib/second",
            "/usr/local/lib/second",
        );
        let mut dep_builder = builder.start_dependency_graph();
        dep_builder
            .insert("root", "1.0.0")
            .with_dependency("first", "1.0.0")
            .with_dependency("second", "1.0.0");
        dep_builder.insert("first", "1.0.0");
        dep_builder.insert("second", "1.0.0");
        let graph = dep_builder.graph("root", "1.0.0");

        let pkg = resolve_package(&graph).unwrap();

        assert_eq!(
            pkg.filesystem,
            vec![
                ResolvedFileSystemMapping {
                    mount_path: PathBuf::from("/usr/local/lib/first"),
                    volume_name: "atom".to_string(),
                    original_path: "/usr/local/lib/first".to_string(),
                    package: builder.get("first", "1.0.0").package_id(),
                },
                ResolvedFileSystemMapping {
                    mount_path: PathBuf::from("/usr/local/lib/second"),
                    original_path: "/usr/local/lib/second".to_string(),
                    volume_name: "atom".to_string(),
                    package: builder.get("second", "1.0.0").package_id(),
                },
                ResolvedFileSystemMapping {
                    mount_path: PathBuf::from("/root"),
                    original_path: "/root".to_string(),
                    volume_name: "atom".to_string(),
                    package: builder.get("root", "1.0.0").package_id(),
                }
            ]
        );
    }

    #[test]
    fn use_fs_mapping_from_dependency() {
        let mut builder = RegistryBuilder::new();
        builder
            .register("root", "1.0.0")
            .with_dependency("dep", "=1.0.0")
            .with_fs_mapping_from_dependency("dep-volume", "/root", "/root", "dep");
        builder.register("dep", "1.0.0");
        let mut dep_builder = builder.start_dependency_graph();
        dep_builder
            .insert("root", "1.0.0")
            .with_dependency("dep", "1.0.0");
        dep_builder.insert("dep", "1.0.0");
        let graph = dep_builder.graph("root", "1.0.0");

        let pkg = resolve_package(&graph).unwrap();

        assert_eq!(
            pkg.filesystem,
            vec![ResolvedFileSystemMapping {
                mount_path: PathBuf::from("/root"),
                original_path: "/root".to_string(),
                volume_name: "dep-volume".to_string(),
                package: builder.get("dep", "1.0.0").package_id(),
            }]
        );
    }
}
