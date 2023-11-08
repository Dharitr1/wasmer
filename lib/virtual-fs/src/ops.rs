//! Common [`FileSystem`] operations.
#![allow(dead_code)] // Most of these helpers are used during testing

use std::{
    collections::VecDeque,
    path::{Component, Path, PathBuf},
};

use futures::future::BoxFuture;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{DirEntry, FileSystem, FsError};

/// Does this item exists?
pub fn exists<F>(fs: &F, path: impl AsRef<Path>) -> bool
where
    F: FileSystem + ?Sized,
{
    fs.metadata(path.as_ref()).is_ok()
}

/// Does this path refer to a directory?
pub fn is_dir<F>(fs: &F, path: impl AsRef<Path>) -> bool
where
    F: FileSystem + ?Sized,
{
    match fs.metadata(path.as_ref()) {
        Ok(meta) => meta.is_dir(),
        Err(_) => false,
    }
}

/// Does this path refer to a file?
pub fn is_file<F>(fs: &F, path: impl AsRef<Path>) -> bool
where
    F: FileSystem + ?Sized,
{
    match fs.metadata(path.as_ref()) {
        Ok(meta) => meta.is_file(),
        Err(_) => false,
    }
}

/// Make sure a directory (and all its parents) exist.
///
/// This is analogous to [`std::fs::create_dir_all()`].
pub fn create_dir_all<F>(fs: &F, path: impl AsRef<Path>) -> Result<(), FsError>
where
    F: FileSystem + ?Sized,
{
    let path = path.as_ref().clean_safely()?;
    if let Some(parent) = path.parent() {
        create_dir_all(fs, parent)?;
    }

    if let Ok(metadata) = fs.metadata(&path) {
        if metadata.is_dir() {
            return Ok(());
        }
        if metadata.is_file() {
            return Err(FsError::BaseNotDirectory);
        }
    }

    fs.create_dir(&path)
}

static WHITEOUT_PREFIX: &str = ".wh.";

/// Creates a white out file which hides it from secondary file systems
pub fn create_white_out<F>(fs: &F, path: impl AsRef<Path>) -> Result<(), FsError>
where
    F: FileSystem + ?Sized,
{
    if let Some(filename) = path.as_ref().file_name() {
        let mut path = path.as_ref().to_owned();
        path.set_file_name(format!("{}{}", WHITEOUT_PREFIX, filename.to_string_lossy()));

        if let Some(parent) = path.parent() {
            create_dir_all(fs, parent).ok();
        }

        fs.new_open_options()
            .create_new(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        Ok(())
    } else {
        Err(FsError::EntryNotFound)
    }
}

/// Removes a white out file from the primary
pub fn remove_white_out<F>(fs: &F, path: impl AsRef<Path>)
where
    F: FileSystem + ?Sized,
{
    if let Some(filename) = path.as_ref().file_name() {
        let mut path = path.as_ref().to_owned();
        path.set_file_name(format!("{}{}", WHITEOUT_PREFIX, filename.to_string_lossy()));
        fs.remove_file(&path).ok();
    }
}

/// Returns true if the path has been hidden by a whiteout file
pub fn has_white_out<F>(fs: &F, path: impl AsRef<Path>) -> bool
where
    F: FileSystem + ?Sized,
{
    if let Some(filename) = path.as_ref().file_name() {
        let mut path = path.as_ref().to_owned();
        path.set_file_name(format!("{}{}", WHITEOUT_PREFIX, filename.to_string_lossy()));
        fs.metadata(&path).is_ok()
    } else {
        false
    }
}

/// Returns true if the path is a whiteout file
pub fn is_white_out(path: impl AsRef<Path>) -> Option<PathBuf> {
    if let Some(filename) = path.as_ref().file_name() {
        if let Some(filename) = filename.to_string_lossy().strip_prefix(WHITEOUT_PREFIX) {
            let mut path = path.as_ref().to_owned();
            path.set_file_name(filename);
            return Some(path);
        }
    }
    None
}

/// Copies the reference of a file from one file system to another
pub fn copy_reference<'a>(
    source: &'a (impl FileSystem + ?Sized),
    destination: &'a (impl FileSystem + ?Sized),
    path: &'a Path,
) -> BoxFuture<'a, Result<(), std::io::Error>> {
    Box::pin(async { copy_reference_ext(source, destination, path, path).await })
}

/// Copies the reference of a file from one file system to another
pub fn copy_reference_ext<'a>(
    source: &'a (impl FileSystem + ?Sized),
    destination: &'a (impl FileSystem + ?Sized),
    from: &Path,
    to: &Path,
) -> BoxFuture<'a, Result<(), std::io::Error>> {
    let from = from.to_owned();
    let to = to.to_owned();
    Box::pin(async move {
        let src = source.new_open_options().read(true).open(from)?;
        let mut dst = destination
            .new_open_options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(to)?;

        dst.copy_reference(src).await?;
        Ok(())
    })
}

/// Asynchronously write some bytes to a file.
///
/// This is analogous to [`std::fs::write()`].
pub async fn write<F>(
    fs: &F,
    path: impl AsRef<Path> + Send,
    data: impl AsRef<[u8]> + Send,
) -> Result<(), FsError>
where
    F: FileSystem + ?Sized,
{
    let path = path.as_ref();
    let data = data.as_ref();

    let mut f = fs
        .new_open_options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;

    f.write_all(data).await?;
    f.flush().await?;

    Ok(())
}

/// Asynchronously read a file's contents into memory.
///
/// This is analogous to [`std::fs::read()`].
pub async fn read<F>(fs: &F, path: impl AsRef<Path> + Send) -> Result<Vec<u8>, FsError>
where
    F: FileSystem + ?Sized,
{
    let mut f = fs.new_open_options().read(true).open(path.as_ref())?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).await?;

    Ok(buffer)
}

/// Asynchronously read a file's contents into memory as a string.
///
/// This is analogous to [`std::fs::read_to_string()`].
pub async fn read_to_string<F>(fs: &F, path: impl AsRef<Path> + Send) -> Result<String, FsError>
where
    F: FileSystem + ?Sized,
{
    let mut f = fs.new_open_options().read(true).open(path.as_ref())?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer).await?;

    Ok(buffer)
}

/// Update a file's modification and access times, creating the file if it
/// doesn't already exist.
pub fn touch<F>(fs: &F, path: impl AsRef<Path> + Send) -> Result<(), FsError>
where
    F: FileSystem + ?Sized,
{
    let _ = fs.new_open_options().create(true).write(true).open(path)?;

    Ok(())
}

/// Recursively iterate over all paths inside a directory, ignoring any
/// errors that may occur along the way.
pub fn walk<F>(fs: &F, path: impl AsRef<Path>) -> Box<dyn Iterator<Item = DirEntry> + '_>
where
    F: FileSystem + ?Sized,
{
    let path = path.as_ref();
    let mut dirs_to_visit: VecDeque<_> = fs
        .read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|result| result.ok())
        .collect();

    Box::new(std::iter::from_fn(move || {
        let next = dirs_to_visit.pop_back()?;

        if let Ok(children) = fs.read_dir(&next.path) {
            dirs_to_visit.extend(children.flatten());
        }

        Some(next)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem_fs::FileSystem as MemFS;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn write() {
        let fs = MemFS::default();

        super::write(&fs, "/file.txt", b"Hello, World!")
            .await
            .unwrap();

        let mut contents = String::new();
        fs.new_open_options()
            .read(true)
            .open("/file.txt")
            .unwrap()
            .read_to_string(&mut contents)
            .await
            .unwrap();
        assert_eq!(contents, "Hello, World!");
    }

    #[tokio::test]
    async fn read() {
        let fs = MemFS::default();
        fs.new_open_options()
            .create(true)
            .write(true)
            .open("/file.txt")
            .unwrap()
            .write_all(b"Hello, World!")
            .await
            .unwrap();

        let contents = super::read_to_string(&fs, "/file.txt").await.unwrap();
        assert_eq!(contents, "Hello, World!");

        let contents = super::read(&fs, "/file.txt").await.unwrap();
        assert_eq!(contents, b"Hello, World!");
    }

    #[tokio::test]
    async fn create_dir_all() {
        let fs = MemFS::default();
        super::write(&fs, "/file.txt", b"").await.unwrap();

        assert!(!super::exists(&fs, "/really/nested/directory"));
        super::create_dir_all(&fs, "/really/nested/directory").unwrap();
        assert!(super::exists(&fs, "/really/nested/directory"));

        // It's okay to create the same directory multiple times
        super::create_dir_all(&fs, "/really/nested/directory").unwrap();

        // You can't create a directory on top of a file
        assert_eq!(
            super::create_dir_all(&fs, "/file.txt").unwrap_err(),
            FsError::BaseNotDirectory
        );
        assert_eq!(
            super::create_dir_all(&fs, "/file.txt/invalid/path").unwrap_err(),
            FsError::BaseNotDirectory
        );
    }

    #[tokio::test]
    async fn touch() {
        let fs = MemFS::default();

        super::touch(&fs, "/file.txt").unwrap();

        assert_eq!(super::read(&fs, "/file.txt").await.unwrap(), b"");
    }
}

/// The Clean trait implements a `clean` method.
pub trait PathClean {
    fn clean_safely(&self) -> crate::Result<PathBuf>;
}

/// PathClean implemented for `Path`
impl PathClean for Path {
    fn clean_safely(&self) -> crate::Result<PathBuf> {
        clean_safely(self)
    }
}

/// PathClean implemented for `PathBuf`
impl PathClean for PathBuf {
    fn clean_safely(&self) -> crate::Result<PathBuf> {
        clean_safely(self)
    }
}

/// The core implementation. It performs the following, lexically:
/// 1. Reduce multiple slashes to a single slash.
/// 2. Eliminate `.` path name elements (the current directory).
/// 3. Eliminate `..` path name elements (the parent directory) and the non-`.` non-`..`, element that precedes them.
/// 4. Eliminate `..` elements that begin a rooted path, that is, replace `/..` by `/` at the beginning of a path.
/// 5. Leave intact `..` elements that begin a non-rooted path.
///
/// If the result of this process is an empty string, return the string `"."`, representing the current directory.
///
/// Code adapted from: https://github.com/danreeves/path-clean/blob/master/src/lib.rs#L50C1-L87C1
pub fn clean_safely<P>(path: P) -> crate::Result<PathBuf>
where
    P: AsRef<Path>,
{
    let mut out = Vec::new();

    for comp in path.as_ref().components() {
        match comp {
            Component::CurDir => (),
            Component::RootDir => {
                if !out.is_empty() {
                    out.push(Component::RootDir);
                }
            }
            Component::ParentDir => match out.last() {
                Some(Component::RootDir) => (),
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                None => {
                    // We tried to go too up the stack
                    return Err(crate::FsError::InvalidInput);
                }
                Some(Component::CurDir)
                | Some(Component::ParentDir)
                | Some(Component::Prefix(_)) => out.push(comp),
            },
            comp => out.push(comp),
        }
    }

    if !out.is_empty() {
        if out[0] == Component::RootDir {
            return Ok(PathBuf::from("."));
        }
        Ok(out.iter().collect())
    } else {
        Ok(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests_clean {
    use super::{clean_safely, PathClean};
    use crate::FsError;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_empty_path_is_current_dir() {
        assert_eq!(clean_safely(""), Ok(PathBuf::from(".")));
    }

    #[test]
    fn test_root_is_current_dir() {
        assert_eq!(clean_safely("/"), Ok(PathBuf::from(".")));
    }

    #[test]
    fn test_clean_paths_dont_change() {
        let tests = vec![
            (".", Ok(PathBuf::from("."))),
            ("..", Err(FsError::InvalidInput)),
            ("/", Ok(PathBuf::from("."))),
        ];

        for test in tests {
            assert_eq!(clean_safely(test.0), test.1);
        }
    }

    #[test]
    fn test_replace_multiple_slashes() {
        let tests = vec![
            ("/", Ok(PathBuf::from("."))),
            ("//", Ok(PathBuf::from("."))),
            ("///", Ok(PathBuf::from("."))),
            (".//", Ok(PathBuf::from("."))),
            ("//..", Err(FsError::InvalidInput)),
            ("..//", Err(FsError::InvalidInput)),
            ("/..//", Err(FsError::InvalidInput)),
            ("/.//./", Ok(PathBuf::from("."))),
            ("././/./", Ok(PathBuf::from("."))),
            ("path//to///thing", Ok(PathBuf::from("path/to/thing"))),
            ("/path//to///thing", Ok(PathBuf::from("path/to/thing"))),
        ];

        for test in tests {
            assert_eq!(clean_safely(test.0), test.1, "test: {}", test.0);
        }
    }

    #[test]
    fn test_eliminate_current_dir() {
        let tests = vec![
            ("./", "."),
            ("/./", "."),
            ("./test", "test"),
            ("./test/./path", "test/path"),
            ("/test/./path/", "test/path"),
            ("test/path/.", "test/path"),
        ];

        for test in tests {
            assert_eq!(clean_safely(test.0), Ok(PathBuf::from(test.1)));
        }
    }

    #[test]
    fn test_eliminate_parent_dir() {
        let tests = vec![
            ("/..", Err(FsError::InvalidInput)),
            ("/../test", Err(FsError::InvalidInput)),
            ("test/..", Ok(PathBuf::from("."))),
            ("test/path/..", Ok(PathBuf::from("test"))),
            ("test/../path", Ok(PathBuf::from("path"))),
            ("/test/../path", Ok(PathBuf::from("path"))),
            ("test/path/../../", Ok(PathBuf::from("."))),
            ("test/path/../../..", Err(FsError::InvalidInput)),
            ("/test/path/../../..", Err(FsError::InvalidInput)),
            ("/test/path/../../../..", Err(FsError::InvalidInput)),
            ("test/path/../../../..", Err(FsError::InvalidInput)),
            (
                "test/path/../../another/path",
                Ok(PathBuf::from("another/path")),
            ),
            (
                "test/path/../../another/path/..",
                Ok(PathBuf::from("another")),
            ),
            ("../test", Err(FsError::InvalidInput)),
            ("../test/", Err(FsError::InvalidInput)),
            ("../test/path", Err(FsError::InvalidInput)),
            ("../test/..", Err(FsError::InvalidInput)),
        ];

        for test in tests {
            assert_eq!(clean_safely(test.0), test.1, "test: {}", test.0);
        }
    }

    #[test]
    fn test_pathbuf_trait() {
        assert_eq!(
            PathBuf::from("/test/../path/").clean_safely(),
            Ok(PathBuf::from("path"))
        );
    }

    #[test]
    fn test_path_trait() {
        assert_eq!(
            Path::new("/test/../path/").clean_safely(),
            Ok(PathBuf::from("path"))
        );
    }

    // #[test]
    // #[cfg(target_os = "windows")]
    // fn test_windows_paths() {
    //     let tests = vec![
    //         ("\\..", "\\"),
    //         ("\\..\\test", "\\test"),
    //         ("test\\..", "."),
    //         ("test\\path\\..\\..\\..", ".."),
    //         ("test\\path/..\\../another\\path", "another\\path"), // Mixed
    //         ("test\\path\\my/path", "test\\path\\my\\path"),      // Mixed 2
    //         ("/dir\\../otherDir/test.json", "/otherDir/test.json"), // User example
    //         ("c:\\test\\..", "c:\\"),                             // issue #12
    //         ("c:/test/..", "c:/"),                                // issue #12
    //     ];

    //     for test in tests {
    //         assert_eq!(clean_safely(test.0), PathBuf::from(test.1));
    //     }
    // }
}
