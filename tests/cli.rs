#![feature(test)]
#![feature(rustc_private)]

#[cfg(test)]
mod tests {


    // extern crate test;
    // extern crate libc;

    // use std::ffi::CString;
    // use std::ffi::c_int;
    // use test_case::test_case;

    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use assert_fs::prelude::*;
    use std::process::Command;

    use tempfile::TempDir;
    use tempfile::NamedTempFile;
    use std::fs::File;

    const STARTING_PATH_STR: &'static str = "/tmp";

    #[test]
    fn new_cli_get_file_by_type_when_type_of_file_is_file() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        let file_name: &'static str = "new_cli_get_file_by_type_when_type_of_file_is_file.txt";
        let temp = assert_fs::TempDir::new().unwrap();
        let input_file = temp.child(file_name);
        input_file.touch()?;
        println!("TEST SETUP: input_file_path: {:#?}", input_file.path());

        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        let val = (format!("{}", temp.to_str().unwrap()));
        println!("{:#?}", val);
        cmd.arg(format!("{}", temp.to_str().unwrap())).arg("--").arg("--type").arg("f");

        // Assert
        let assertion = cmd.assert().try_success()?;
        assertion.try_stdout(predicate::str::contains(file_name))?;

        //Teardown
        Ok(())
    }
    
    #[test]
    fn cli_get_file_by_type_when_type_of_file_is_file() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert!(std::env::set_current_dir(STARTING_PATH_STR).is_ok());
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        assert_fs::TempDir::new().unwrap();
        let file = NamedTempFile::new()?;
        let file_path = file.path().to_str().unwrap().to_string();
        
        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        cmd.arg(STARTING_PATH_STR).arg("--").arg("--type").arg("f");

        // Assert
        let assertion = cmd.assert().try_success()?;
        assertion.stdout(predicate::str::contains(file_path));

        //Teardown
        drop(file);
        Ok(())
    }

    #[test]
    fn cli_get_file_by_type_when_type_of_file_is_dir() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert!(std::env::set_current_dir(STARTING_PATH_STR).is_ok());
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let directory = TempDir::new()?;
        let directory_path = directory.path().to_str().unwrap();

        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        cmd.arg(STARTING_PATH_STR).arg("--").arg("--type").arg("d");

        // Assert
        let assertion = cmd.assert().try_success()?;
        assertion.stdout(predicate::str::contains(directory_path));

        // Teardown
        directory.close()?;
        Ok(())
    }

    #[test]
    fn cli_get_file_by_type_when_type_of_file_is_symlink_and_symlink_setting_is_not_set_to_follow() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert!(std::env::set_current_dir(STARTING_PATH_STR).is_ok());
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

        let current_directory = TempDir::new()?;
        let directory_of_link = TempDir::new()?;
        let working_directory_before_test = std::env::current_dir().unwrap();
        assert!(std::env::set_current_dir(current_directory.path()).is_ok());

        let original_file_path = current_directory.path().join("get_file_by_type_when_type_of_file_is_symlink_and_symlink_setting_is_not_set_to_follow.txt");
        let original_file = File::create(original_file_path.clone())?;

        let directory_of_link_path = directory_of_link.path().join("symlink");
        std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;

        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        cmd.arg("-P").arg(STARTING_PATH_STR).arg("--").arg("--type").arg("s");

        // Assert
        let assertion = cmd.assert().try_success()?;
        assertion.try_stdout(predicate::str::contains(directory_of_link_path.to_str().unwrap()).not())?;

        // Teardown
        current_directory.close()?;
        directory_of_link.close()?;
        let _ = std::env::set_current_dir(working_directory_before_test)?;
        drop(original_file);
        Ok(())
    }

    #[test]
    fn cli_do_not_get_file_by_type_when_type_of_file_is_symlink_and_symlink_setting_is_set_to_follow() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert!(std::env::set_current_dir(STARTING_PATH_STR).is_ok());
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

        let current_directory = TempDir::new()?;
        let directory_of_link = TempDir::new()?;
        let working_directory_before_test = std::env::current_dir().unwrap();
        assert!(std::env::set_current_dir(current_directory.path()).is_ok());

        let original_file_path = current_directory.path().join("do_not_get_file_by_type_when_type_of_file_is_symlink_and_symlink_setting_is_set_to_follow.txt");
        let original_file = File::create(original_file_path.clone())?;

        let directory_of_link_path = directory_of_link.path().join("symlink");
        std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;

        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        cmd.arg("-L").arg(STARTING_PATH_STR).arg("--").arg("--type").arg("s");

        // Assert
        let assertion = cmd.assert().try_success()?;
        assertion.try_stdout(predicate::str::contains(directory_of_link_path.to_str().unwrap()).not())?;

        // Teardown
        current_directory.close()?;
        directory_of_link.close()?;
        let _ = std::env::set_current_dir(working_directory_before_test)?;
        drop(original_file);
        Ok(())
    }

    #[test]
    fn cli_get_file_when_multiple_types_are_provided() -> Result<(), Box<dyn std::error::Error>> {
        // Assert
        assert!(std::env::set_current_dir(STARTING_PATH_STR).is_ok());
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

        let file = NamedTempFile::new()?;
        let file_path = file.path().to_str().unwrap().to_string();
        
        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        cmd.arg(STARTING_PATH_STR).arg("--").arg("--type").arg("bcf");

        // Assert
        let assertion = cmd.assert().try_success()?;
        assertion.stdout(predicate::str::contains(file_path));

        // Teardown
        drop(file);
        Ok(())
    }

    #[test]
    fn cli_find_file_in_same_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert!(std::env::set_current_dir(STARTING_PATH_STR).is_ok());
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

        let file = NamedTempFile::new()?;
        let file_path = file.path().to_str().unwrap().to_string();
        let file_name = file.path().file_name().unwrap().to_str().unwrap();
        
        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        cmd.arg(STARTING_PATH_STR).arg("--").arg("--name").arg(file_name);

        // Assert
        cmd.assert().success().stdout(predicate::str::contains(file_path));

        // Teardown
        drop(file);
        Ok(())
    }

    #[test]
    fn cli_find_file_in_child_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert!(std::env::set_current_dir(STARTING_PATH_STR).is_ok());
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;

        // Create a file in that new directory with the name specified in FILE_NAME
        const FILE_NAME: &'static str = "cli_find_file_in_child_directory.txt";
        let file_path = directory.path().join(FILE_NAME);

        // Create a file inside of the newly created directory
        let file = File::create(file_path.clone())?;
        
        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        cmd.arg(STARTING_PATH_STR).arg("--").arg("--name").arg(FILE_NAME);

        // Assert
        cmd.assert().success().stdout(predicate::str::contains(file_path.to_str().unwrap().to_string()));

        // Teardown
        drop(file);
        directory.close()?;
        Ok(())
    }

    #[test]
    fn cli_does_not_find_file_in_child_directory_when_max_depth_is_set_to_zero() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange 
        assert!(std::env::set_current_dir(STARTING_PATH_STR).is_ok());
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;
        let file_path = directory.path().join("find_file_in_child_directory.txt");
        // Create a file inside of the newly created directory
        let tmp_file = File::create(file_path.clone())?;

        // Act
        let mut cmd = Command::cargo_bin("rfind")?;
        cmd.arg(STARTING_PATH_STR).arg("--").arg("--name").arg(file_path.clone()).arg("--maxdepth").arg("0");

        // Assert
        cmd.assert().success().stdout(predicate::str::contains(file_path.to_str().unwrap()).not());

        // Teardown
        drop(tmp_file);
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
        cmd.arg("-L").arg(STARTING_PATH_STR).arg("--").arg("--name").arg("symlink");

        // Assert
        cmd.assert().success().stdout(predicate::str::contains(directory_of_link_path.to_str().unwrap()).not());

        // Teardown
        current_directory.close()?;
        directory_of_link.close()?;
        let _ = std::env::set_current_dir(working_directory_before_test)?;
        drop(original_file);
        Ok(())
    }
}
