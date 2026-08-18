use std::path::{Path, PathBuf};
use std::{fs, io};

fn main() {
    // Copy dsh dependencies from the pnpm virtual store into
    // resources/dsh/node_modules/ so the bundled app is self-contained.
    // Node.js itself is handled separately via Tauri's externalBin mechanism
    // (binaries/node), so we do NOT copy it here.
    copy_dsh_node_modules();

    tauri_build::build()
}

/// Copy packages from the pnpm virtual store into resources/dsh/node_modules/.
/// Only copies what dsh needs — does not copy Node.js itself (handled by externalBin).
fn copy_dsh_node_modules() {
    let pnpm_store = PathBuf::from("node_modules/.pnpm");
    let target = PathBuf::from("resources/dsh/node_modules");

    // If pnpm virtual store doesn't exist, skip silently.
    // User needs to run `pnpm install` before building.
    if !pnpm_store.exists() {
        eprintln!(
            "[deepseek-desktop] pnpm virtual store not found at {:?}, skipping node_modules copy. Run `pnpm install` first.",
            pnpm_store
        );
        return;
    }

    // Skip if target already populated
    if target.exists() {
        let count = target.read_dir().map(|d| d.count()).unwrap_or(0);
        if count > 0 {
            eprintln!(
                "[deepseek-desktop] resources/dsh/node_modules/ already populated, skipping copy."
            );
            return;
        }
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

            // Skip bare scope dirs like "@deepseek-ai" — recurse into them
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
                // Scoped package: @scope/name -> resources/dsh/node_modules/@scope/name/
                let parts: Vec<&str> = pkg_name.split('/').collect();
                if parts.len() < 3 {
                    continue;
                }
                target.join(parts[0]).join(parts[1])
            } else {
                target.join(pkg_name)
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
        "[deepseek-desktop] copied {} packages into resources/dsh/node_modules/ ({:?})",
        copied,
        target.display()
    );
}

/// Recursively copy a directory.
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
