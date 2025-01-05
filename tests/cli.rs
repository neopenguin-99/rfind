use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use tempfile::TempDir;
use tempfile::NamedTempFile;
use std::fs::File;

#[test]
fn find_file_in_same_directory() -> Result<(), Box<dyn std::error::Error>> {
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
fn does_not_find_file_in_child_directory_when_max_depth_is_set_to_zero() -> Result<(), Box<dyn std::error::Error>> {
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
    directory.close()?;
    Ok(())
}
