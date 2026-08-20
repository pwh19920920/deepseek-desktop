use std::path::{Path, PathBuf};
use std::{fs, io};

fn main() {
    // Bundle dsh and its node_modules dependencies into the Tauri resource directory.
    copy_dsh_source();
    copy_dsh_node_modules();
    prune_dsh_node_modules();
    pack_dsh_node_modules();

    tauri_build::build()
}

/// Copy the dsh kernel source from node_modules into ../resources/dsh/.
/// Skips the node_modules/ subdirectory — that's handled by copy_dsh_node_modules().
fn copy_dsh_source() {
    let src = PathBuf::from("../node_modules/@deepseek-ai/dsh");
    let dest = PathBuf::from("../resources/dsh");

    if !src.exists() {
        eprintln!(
            "[deepseek-desktop] dsh package not found at {:?}. Run `pnpm install` first.",
            src
        );
        return;
    }

    // Avoid re-copying if already populated
    if dest.join("lib").join("bin.js").exists() {
        return;
    }

    fs::create_dir_all(&dest).unwrap_or_else(|e| {
        eprintln!("[deepseek-desktop] failed to create {:?}: {}", dest, e);
        std::process::exit(1);
    });

    for entry in fs::read_dir(&src).expect("failed to read dsh package") {
        let entry = match entry {
            Ok(e) => e.path(),
            Err(_) => continue,
        };
        let name = entry.file_name().unwrap_or_default();
        // Skip node_modules — will be populated separately
        if name == "node_modules" {
            continue;
        }
        let target = dest.join(name);
        let ty = match entry.metadata() {
            Ok(m) => m.file_type(),
            Err(_) => continue,
        };
        if ty.is_dir() {
            if let Err(e) = copy_dir_all(&entry, &target) {
                eprintln!(
                    "[deepseek-desktop] warning: failed to copy dir {:?}: {}",
                    entry, e
                );
            }
        } else if let Err(e) = fs::copy(&entry, &target) {
            eprintln!(
                "[deepseek-desktop] warning: failed to copy file {:?}: {}",
                entry, e
            );
        }
    }

    eprintln!("[deepseek-desktop] dsh source copied to {:?}", dest);
}

/// Copy packages from the pnpm virtual store into ../resources/dsh/node_modules/.
fn copy_dsh_node_modules() {
    let pnpm_store = PathBuf::from("../../node_modules/.pnpm");
    let target = PathBuf::from("../resources/dsh/node_modules");

    if !pnpm_store.exists() {
        eprintln!(
            "[deepseek-desktop] pnpm virtual store not found at {:?}, skipping node_modules copy.",
            pnpm_store
        );
        return;
    }

    if target.join("dsh").exists()
        || target.join(".package-lock.json").exists()
        || target
            .parent()
            .unwrap()
            .join("node_modules.tar.gz")
            .exists()
    {
        eprintln!(
            "[deepseek-desktop] ../resources/dsh/node_modules/ already populated, skipping copy."
        );
        return;
    }

    fs::create_dir_all(&target).unwrap_or_else(|e| {
        eprintln!("[deepseek-desktop] failed to create {:?}: {}", target, e);
        std::process::exit(1);
    });

    let mut copied = 0usize;

    for store_dir in fs::read_dir(&pnpm_store).expect("failed to read pnpm store") {
        let store_dir = match store_dir {
            Ok(d) => d.path(),
            Err(_) => continue,
        };
        if !store_dir.is_dir() {
            continue;
        }

        let pkg_root = store_dir.join("node_modules");
        if !pkg_root.exists() {
            continue;
        }

        for pkg_entry in fs::read_dir(&pkg_root).expect("failed to read node_modules") {
            let pkg = match pkg_entry {
                Ok(p) => p.path(),
                Err(_) => continue,
            };
            if !pkg.is_dir() {
                continue;
            }

            let pkg_name = match pkg.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };

            // Recurse into scope directories (e.g. @deepseek-ai)
            if pkg_name.starts_with('@') && !pkg_name.contains('/') {
                if let Ok(entries) = fs::read_dir(&pkg) {
                    for sub_entry in entries {
                        let sub_pkg = match sub_entry {
                            Ok(e) => e.path(),
                            Err(_) => continue,
                        };
                        if !sub_pkg.is_dir() {
                            continue;
                        }
                        let sub_name = match sub_pkg.file_name() {
                            Some(n) => n.to_string_lossy().to_string(),
                            None => continue,
                        };
                        let dest = target.join(&pkg_name).join(&sub_name);
                        if dest.exists() {
                            continue;
                        }
                        if let Err(e) = copy_dir_all(&sub_pkg, &dest) {
                            eprintln!(
                                "[deepseek-desktop] warning: failed to copy {:?}: {}",
                                sub_pkg, e
                            );
                        } else {
                            copied += 1;
                        }
                    }
                }
                continue;
            }

            let dest = if pkg_name.starts_with('@') {
                let parts: Vec<&str> = pkg_name.split('/').collect();
                if parts.len() < 3 {
                    continue;
                }
                target.join(parts[0]).join(parts[1])
            } else {
                target.join(&pkg_name)
            };

            if dest.exists() {
                continue;
            }

            if let Err(e) = copy_dir_all(&pkg, &dest) {
                eprintln!(
                    "[deepseek-desktop] warning: failed to copy {:?}: {}",
                    pkg, e
                );
            } else {
                copied += 1;
            }
        }
    }

    eprintln!(
        "[deepseek-desktop] copied {} packages into ../resources/dsh/node_modules/",
        copied
    );
}

/// Remove dev-only packages and strip unnecessary platform binaries from the
/// bundled node_modules to reduce bundle size.
fn prune_dsh_node_modules() {
    let nm = PathBuf::from("../resources/dsh/node_modules");
    if !nm.exists() {
        return;
    }

    // Packages that are definitely build-only and not needed at runtime.
    // Note: @babel is kept because cordis-plugin-hmr needs @babel/code-frame.
    let removable = ["typescript", "@esbuild", "vite", "rollup", "caniuse-lite"];

    for name in &removable {
        let path = nm.join(name);
        if path.exists() {
            let size = dir_size(&path);
            rm_rf(&path);
            eprintln!("[deepseek-desktop] pruned {} ({})", name, format_size(size));
        }
    }

    // Strip unnecessary platform prebuilds from node-pty.
    // Only keep the prebuild matching the current build target.
    let node_pty_prebuilds = nm.join("node-pty").join("prebuilds");
    if node_pty_prebuilds.exists() {
        let target = std::env::var("TARGET").unwrap_or_default();
        let keep = match target.as_str() {
            "aarch64-apple-darwin" => "darwin-arm64",
            "x86_64-apple-darwin" => "darwin-x64",
            "x86_64-pc-windows-msvc" => "win32-x64",
            "aarch64-pc-windows-msvc" => "win32-arm64",
            "x86_64-unknown-linux-gnu" => "linux-x64",
            "aarch64-unknown-linux-gnu" => "linux-arm64",
            _ => "", // unknown target, keep all
        };
        if !keep.is_empty() {
            if let Ok(entries) = fs::read_dir(&node_pty_prebuilds) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if !name.starts_with(keep) {
                        let size = dir_size(&entry.path());
                        rm_rf(&entry.path());
                        eprintln!(
                            "[deepseek-desktop] pruned node-pty prebuild: {} ({})",
                            name,
                            format_size(size)
                        );
                    }
                }
            }
        }
    }

    // Remove .md, .d.ts, test files, etc.
    remove_junk(&nm);
}

/// Recursively remove a file or directory.
fn rm_rf(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else {
        let _ = fs::remove_file(path);
    }
}

/// Calculate the total size of a directory in bytes.
fn dir_size(path: &Path) -> u64 {
    fn walk(dir: &Path) -> io::Result<u64> {
        let mut total = 0u64;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                total += walk(&entry.path())?;
            } else {
                total += entry.metadata()?.len();
            }
        }
        Ok(total)
    }
    walk(path).unwrap_or(0)
}

/// Format bytes as a human-readable string.
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1}{}", size, UNITS[unit])
}

/// Remove junk files (.md, .d.ts, tests, etc.) from the bundled node_modules.
fn remove_junk(nm: &Path) {
    let mut removed = 0u64;
    let patterns = &["CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md"];

    if let Ok(entries) = fs::read_dir(nm) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Remove known junk files at the root of each package
            for pattern in patterns {
                let junk = path.join(pattern);
                if junk.exists() && junk.is_file() {
                    if let Ok(m) = junk.metadata() {
                        removed += m.len();
                    }
                    let _ = fs::remove_file(&junk);
                }
            }
        }
    }

    if removed > 0 {
        eprintln!(
            "[deepseek-desktop] removed {} in documentation files",
            format_size(removed)
        );
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path().as_ref(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Pack the pruned node_modules into a single tar.gz archive, then remove the
/// original directory. This dramatically reduces the number of files in the
/// bundle, speeding up installer extraction on Windows.
fn pack_dsh_node_modules() {
    let nm = PathBuf::from("../resources/dsh/node_modules");
    let tarball = PathBuf::from("../resources/dsh/node_modules.tar.gz");

    if !nm.exists() {
        return;
    }
    if tarball.exists() {
        eprintln!("[deepseek-desktop] node_modules.tar.gz already exists, skipping pack.");
        return;
    }

    eprintln!("[deepseek-desktop] packing node_modules into node_modules.tar.gz ...");

    let file = match fs::File::create(&tarball) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[deepseek-desktop] failed to create tarball: {}", e);
            return;
        }
    };
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut archive = tar::Builder::new(encoder);

    // Walk the node_modules directory and add each entry to the archive
    fn add_dir<W: io::Write>(
        archive: &mut tar::Builder<W>,
        path: &Path,
        prefix: &Path,
    ) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry.file_name();
            // Path within the archive (relative to node_modules/)
            let archive_path = prefix.join(&entry_name);
            let ty = entry.file_type()?;
            if ty.is_dir() {
                archive.append_dir(&archive_path, &entry_path)?;
                add_dir(archive, &entry_path, &archive_path)?;
            } else {
                archive.append_file(&archive_path, &mut fs::File::open(&entry_path)?)?;
            }
        }
        Ok(())
    }

    if let Err(e) = add_dir(&mut archive, &nm, &Path::new(".")) {
        eprintln!("[deepseek-desktop] failed to pack node_modules: {}", e);
        let _ = fs::remove_file(&tarball);
        return;
    }

    match archive.finish() {
        Ok(_) => {
            let size = dir_size(&tarball);
            eprintln!(
                "[deepseek-desktop] packed node_modules.tar.gz ({})",
                format_size(size)
            );
            // Remove the original directory
            rm_rf(&nm);
            eprintln!("[deepseek-desktop] removed original node_modules directory");
        }
        Err(e) => {
            eprintln!("[deepseek-desktop] failed to finalize tarball: {}", e);
            let _ = fs::remove_file(&tarball);
        }
    }
}
