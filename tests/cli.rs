use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use tempfile::TempDir;
use tempfile::NamedTempFile;
use std::fs::File;

#[test]
fn cli_find_file_in_same_directory() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    assert!(std::env::set_current_dir("/tmp").is_ok());
    assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

    let file = NamedTempFile::new()?;
    let file_path = file.path().file_name().unwrap().to_str().unwrap();
    
    // Act
    let mut cmd = Command::cargo_bin("rfind")?;
    cmd.arg("--name").arg(file_path);

    // Assert
    cmd.assert().success().stdout(predicate::str::contains(file_path));

    // Teardown
    drop(file);
    Ok(())
}

#[test]
fn cli_does_not_find_file_in_child_directory_when_max_depth_is_set_to_zero() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange 
    assert!(std::env::set_current_dir("/tmp").is_ok());
    assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

    // Create a directory inside of `env::temp_dir()`
    let directory = TempDir::new()?;
    let file_path = directory.path().join("find_file_in_child_directory.txt");
    // Create a file inside of the newly created directory
    let tmp_file = File::create(file_path.clone())?;

    // Act
    let mut cmd = Command::cargo_bin("rfind")?;
    cmd.arg("--name").arg(file_path.clone()).arg("--maxdepth").arg("0");

    // Assert
    cmd.assert().success().stdout(predicate::str::contains(file_path.to_str().unwrap()).not());

    // Teardown
    drop(tmp_file);
    // todo figure out some way of removing the symlink
    directory.close()?;
    Ok(())
}

#[test]
fn cli_follows_symlink_when_set_to_follow() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let current_directory = TempDir::new()?;
    let directory_of_link = TempDir::new()?;
    let working_directory_before_test = std::env::current_dir().unwrap();
    assert!(std::env::set_current_dir(current_directory.path()).is_ok());

    let original_file_path = current_directory.path().join("does_not_follow_symbolic_links_by_default.txt");
    let original_file = File::create(original_file_path.clone())?;

    let directory_of_link_path = directory_of_link.path().join("symlink");
    std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;

    // Act
    let mut cmd = Command::cargo_bin("rfind")?;
    cmd.arg("-L").arg("--name").arg("symlink");

    // Assert
    cmd.assert().success().stdout(predicate::str::contains(directory_of_link_path.to_str().unwrap()).not());

    // Teardown
    current_directory.close()?;
    directory_of_link.close()?;
    let _ = std::env::set_current_dir(working_directory_before_test)?;
    drop(original_file);
    Ok(())
}
